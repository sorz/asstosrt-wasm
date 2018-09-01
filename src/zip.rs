use std::io::{self, Write, Read, Seek, SeekFrom};
use crc::{crc32, Hasher32};

const LOCAL_FILE_HEADER_SIGNATURE: &'static [u8] = b"\x50\x4b\x03\x04";
const CENTRAL_FILE_HEADER_SIGNATURE: &'static [u8] = b"\x50\x4b\x01\x02";
const EOF_CENTRAL_FILE_HEADER_SIGNATURE: &'static [u8] = b"\x50\x4b\x05\x06";
const VERSION_NEED_TO_EXTRACT_DEFAULT: &'static [u8] = b"\x00\x00";
const VERSION_MADE_BY: &'static [u8] = b"\x00\x3f";  // 6.3
const GENERAL_PURPOSE_BIT_FLAG: &'static [u8] = b"\x00\x00";
const COMPRESSION_METHOD_STORE: &'static [u8] = b"\x00\x00";
const LENGTH_ZERO: &'static [u8] = b"\x00\x00";
const INTERNAL_FILE_ATTRS: &'static [u8] = b"\x10\x00"; // text file
const EXTERNAL_FILE_ATTRS: &'static [u8] = b"\x00\x00\x00\x00";
const UNICODE_PATH_EXTRA_FIELD: &'static [u8] = b"\x75\x70";
const UNICODE_PATH_VERSION: &'static [u8] = b"\x01";


pub struct ZipWriter<W> {
    writer: W,
    files: Vec<FileEntry>,
    cursor: u64,
}

struct FileEntry {
    offset: u64,
    filename: Box<str>,
    size: u64,
    crc32: u32,
}

#[derive(Debug, PartialEq)]
enum FileHeader {
    Local,
    Central,
}

struct Utf8PathField<'a> {
    path: &'a str,
}

impl<'a> Utf8PathField<'a> {
    fn new(path: &'a str) -> Self {
        Utf8PathField { path }
    }

    fn into_bytes(self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(self.path.len() + 9);
        buf.write(UNICODE_PATH_EXTRA_FIELD).unwrap();
        buf.write(&((self.path.len() + 5) as u16).to_le_bytes()).unwrap();
        buf.write(UNICODE_PATH_VERSION).unwrap();

        let mut digest = crc32::Digest::new(crc32::IEEE);
        digest.write(self.path.as_bytes());
        buf.write(&digest.sum32().to_le_bytes()).unwrap();

        buf.write(self.path.as_bytes()).unwrap();
        buf.into_boxed_slice()
    }
}

impl FileHeader {
    fn signature(&self) -> &'static [u8] {
        match self {
            FileHeader::Local => LOCAL_FILE_HEADER_SIGNATURE,
            FileHeader::Central => CENTRAL_FILE_HEADER_SIGNATURE,
        }
    }
}

impl FileEntry {
    fn new(offset: u64, filename: Box<str>, size: u64, crc32: u32) -> Self {
        FileEntry { offset, filename, size, crc32 }
    }

    fn write_header<W>(&self, write: &mut W, header: FileHeader) -> io::Result<usize>
    where W: Write {
        let mut n = 0;
        n += write.write(header.signature())?;
        if header == FileHeader::Central {
            n += write.write(VERSION_MADE_BY)?;
        }
        n += write.write(VERSION_NEED_TO_EXTRACT_DEFAULT)?;
        n += write.write(GENERAL_PURPOSE_BIT_FLAG)?;
        n += write.write(COMPRESSION_METHOD_STORE)?;
        n += write.write(b"\x00\x00\x00\x00")?; // time & date 
        n += write.write(&self.crc32.to_le_bytes())?;
        let size_bytes = (self.size as u32).to_le_bytes();
        n += write.write(&size_bytes)?;
        n += write.write(&size_bytes)?;
        n += write.write(&(self.filename.len() as u16).to_le_bytes())?;
        let extra = Utf8PathField::new(&self.filename).into_bytes();
        n += write.write(&(extra.len() as u16).to_le_bytes())?;
        if header == FileHeader::Central {
            n += write.write(LENGTH_ZERO)?; // file comment
            n += write.write(LENGTH_ZERO)?; // disk number
            n += write.write(INTERNAL_FILE_ATTRS)?;
            n += write.write(EXTERNAL_FILE_ATTRS)?;
            n += write.write(&(self.offset as u32).to_le_bytes())?;
        }        
        n += write.write(self.filename.as_bytes())?;
        n += write.write(&extra)?;
        Ok(n)
    }
}

impl<W> ZipWriter<W>
where W: Write + Seek {
    pub fn new(writer: W) -> Self {
        ZipWriter {
            writer,
            files: Vec::new(),
            cursor: 0,
        }
    }

    pub fn write_file<R>(&mut self, filename: &str,
                         content: R) -> io::Result<()>
    where R: Read {
        // write local header
        let filename = filename.to_owned().into_boxed_str();
        let mut file = FileEntry::new(self.cursor, filename, 0, 0);
        self.cursor += file.write_header(&mut self.writer,
                                         FileHeader::Local)? as u64;

        // write file content
        let mut content = Crc32Reader::new(content);
        file.size = io::copy(&mut content, &mut self.writer)?;
        file.crc32 = content.sum32();
        self.cursor += file.size;

        // update header
        self.writer.seek(SeekFrom::Start(file.offset))?;
        file.write_header(&mut self.writer, FileHeader::Local)?;
        self.writer.seek(SeekFrom::Start(self.cursor))?;

        self.files.push(file);
        Ok(())
    }

    pub fn close(self) -> io::Result<()> {
        let ZipWriter { mut writer, files, cursor } = self;

        let entries_len = (files.len().to_le() as u16).to_le_bytes();
        let mut len = 0;
        for file in files {
            len += file.write_header(&mut writer, FileHeader::Central)?;
        }

        writer.write(EOF_CENTRAL_FILE_HEADER_SIGNATURE)?;
        writer.write(LENGTH_ZERO)?;  // number of this disk
        writer.write(&1u16.to_le_bytes())?;  // disk w/ central dir
        writer.write(&entries_len)?;  // in the central dir on this disk
        writer.write(&entries_len)?;  // total in the central dir
        writer.write(&(len as u32).to_le_bytes())?;
        writer.write(&(cursor as u32).to_le_bytes())?;
        writer.write(LENGTH_ZERO)?;  // zip file comment
        Ok(())
    }
}

struct Crc32Reader<R> {
    internal: R,
    digest: crc32::Digest,
}

impl<R: Read> Crc32Reader<R> {
    fn new(internal: R) -> Self {
        Crc32Reader {
            internal,
            digest: crc32::Digest::new(crc32::IEEE),
        }
    }

    fn sum32(&self) -> u32 {
        self.digest.sum32()
    }
}

impl<R: Read> Read for Crc32Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.internal.read(buf)?;
        self.digest.write(&buf[0..len]);
        Ok(len)
    }
}

