use std::{
    ffi::OsStr,
    io::{Cursor, Read},
    path::PathBuf,
};

use js_sys::Uint8Array;
use web_sys::{File, FileReaderSync};
use zip::{ZipArchive, result::ZipError};

use crate::FileWrap;

use super::{ConvertError, FILE_SIZE_LIMIT};

pub(crate) trait ReadToVec {
    fn read_to_vec(&self, file: &File) -> Result<Vec<u8>, ConvertError>;
}

impl ReadToVec for FileReaderSync {
    fn read_to_vec(&self, file: &File) -> Result<Vec<u8>, ConvertError> {
        let array = self.read_as_array_buffer(file)?;
        let mut buf = vec![0u8; array.byte_length().try_into().unwrap()];
        Uint8Array::new(&array).copy_to(&mut buf);
        Ok(buf)
    }
}

#[derive(Debug)]
struct ZipIterator {
    zip: ZipArchive<Cursor<Vec<u8>>>,
    file_idx: usize,
}

impl ZipIterator {
    fn new(buf: Vec<u8>) -> Result<Self, ConvertError> {
        Ok(Self {
            zip: ZipArchive::new(Cursor::new(buf))?,
            file_idx: 0,
        })
    }
}

impl Iterator for ZipIterator {
    type Item = Result<(PathBuf, Vec<u8>), ConvertError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.file_idx < self.zip.len() {
            let file = self.zip.by_index(self.file_idx);
            self.file_idx += 1;
            let mut file = match file {
                Ok(file) => file,
                Err(err) => return Some(Err(err.into())),
            };
            let path = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };
            match path.extension().and_then(OsStr::to_str) {
                Some("ass") | Some("ssa") => (),
                _ => {
                    log::info!("skip file {:?}", path);
                    continue;
                }
            }
            let size = file.size().try_into().unwrap_or(usize::MAX);
            if size > FILE_SIZE_LIMIT {
                return Some(Err(ConvertError::TooLarge {
                    size,
                    limit: FILE_SIZE_LIMIT,
                }));
            }
            let mut buf = Vec::new();
            return match file.read_to_end(&mut buf) {
                Ok(_) => Some(Ok((path, buf))),
                Err(err) => Some(Err(ZipError::Io(err).into())),
            };
        }
        None
    }
}

#[derive(Debug)]
pub(crate) struct FileWalk {
    reader: FileReaderSync,
    files: Vec<FileWrap>,
    file_idx: usize,
    zip: Option<ZipIterator>,
}

impl FileWalk {
    pub(crate) fn new(files: Vec<FileWrap>, reader: FileReaderSync) -> Self {
        Self {
            reader,
            files,
            file_idx: 0,
            zip: None,
        }
    }
}

impl Iterator for FileWalk {
    type Item = Result<(PathBuf, Vec<u8>), ConvertError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.file_idx < self.files.len() {
            match &mut self.zip {
                Some(zip) => {
                    // walk inside the zip file
                    match zip.next() {
                        Some(Ok((path, file))) => {
                            let path = if self.files.len() > 1 {
                                // TODO: move base path creation out of the loop
                                let mut base: PathBuf = self.files[self.file_idx].0.name().into();
                                base.set_extension("");
                                base.push(path);
                                base
                            } else {
                                path
                            };
                            return Some(Ok((path, file)));
                        }
                        Some(Err(err)) => return Some(Err(err)),
                        None => {
                            // end of zip, read next file
                            self.zip = None;
                            self.file_idx += 1;
                        }
                    }
                }
                None => {
                    let file = &self.files[self.file_idx];
                    let name: PathBuf = file.0.name().into();
                    match name.extension().and_then(OsStr::to_str) {
                        Some("zip") => {
                            self.zip =
                                match self.reader.read_to_vec(&file.0).and_then(ZipIterator::new) {
                                    Ok(zip) => Some(zip),
                                    Err(err) => return Some(Err(err)),
                                };
                        }
                        Some("ass") | Some("ssa") => match self.reader.read_to_vec(&file.0) {
                            Ok(buf) => {
                                self.file_idx += 1;
                                return Some(Ok((name, buf)));
                            }
                            Err(err) => return Some(Err(err)),
                        },
                        _ => {
                            log::info!("skip file {:?}", name);
                            self.file_idx += 1;
                        }
                    }
                }
            }
        }
        None
    }
}
