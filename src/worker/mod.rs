mod subtitle;
mod walk;

use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE};
use futures::channel::oneshot::Canceled;
use gloo_net::http::Request;
use js_sys::{Array, Uint8Array};
use serde::{Deserialize, Serialize};
use simplecc::Dict;
use std::{
    borrow::Cow,
    collections::HashSet,
    io::{Cursor, Write},
    ops::AddAssign,
};
use thiserror::Error;
use walk::{FileWalk, ReadToVec};
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, File, FileReaderSync, Url};
use zip::{
    CompressionMethod,
    result::ZipError,
    write::{SimpleFileOptions, ZipWriter},
};

use crate::{FileWrap, Options, TaskRequest, TaskResult};
pub(crate) use subtitle::FormatError;

pub(crate) const FILE_SIZE_LIMIT: usize = 100 * 1024 * 1024;
const MIME_SRT: &str = "text/srt";
const MIME_ZIP: &str = "application/zip";

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConvertError {
    #[error("empty file list")]
    NoFile,
    #[error("file too large ({size} > {limit} bytes)")]
    TooLarge { size: usize, limit: usize },
    #[error("failed to fetch OpenCC dict: {0}")]
    FetchDict(String),
    #[error("unknown encoding label `{0}`")]
    EncodingLabel(String),
    #[error("utf-16 as output is not supported")]
    Utf16Output,
    #[error("failed to guess input encoding")]
    EncodingDetect,
    #[error("ass format error: {0}")]
    Format(#[from] FormatError),
    #[error("canceled")]
    Canceled,
    #[error("zip file error: {0}")]
    Zip(String),
    #[error("{name}: {msg}")]
    JsError { name: String, msg: String },
}

impl From<JsValue> for ConvertError {
    fn from(value: JsValue) -> Self {
        match value.dyn_into::<js_sys::Error>() {
            Ok(err) => Self::JsError {
                name: err.name().into(),
                msg: err.message().into(),
            },
            Err(value) => Self::JsError {
                name: value
                    .js_typeof()
                    .as_string()
                    .unwrap_or("unknown".to_string()),
                msg: value.as_string().unwrap_or("unknown".to_string()),
            },
        }
    }
}

impl From<Canceled> for ConvertError {
    fn from(_: Canceled) -> Self {
        Self::Canceled
    }
}

impl From<ZipError> for ConvertError {
    fn from(value: ZipError) -> Self {
        Self::Zip(value.to_string())
    }
}

// gloo_net::Error did not impl serde, we convert it to String manually
impl From<gloo_net::Error> for ConvertError {
    fn from(value: gloo_net::Error) -> Self {
        let msg = match value {
            gloo_net::Error::JsError(err) => format!("[{}] {}", err.name, err.message),
            gloo_net::Error::SerdeError(err) => err.to_string(),
            gloo_net::Error::GlooError(msg) => msg,
        };
        Self::FetchDict(msg)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ConvertMeta {
    pub(crate) input_encoding: HashSet<String>,
    pub(crate) output_encoding: HashSet<String>,
    pub(crate) decode_error: bool,
    pub(crate) encode_error: bool,
}

impl AddAssign for ConvertMeta {
    fn add_assign(&mut self, rhs: Self) {
        self.input_encoding.extend(rhs.input_encoding);
        self.output_encoding.extend(rhs.output_encoding);
        self.decode_error |= rhs.decode_error;
        self.encode_error |= rhs.encode_error;
    }
}

impl ConvertMeta {
    pub(crate) fn has_error(&self) -> bool {
        self.decode_error || self.encode_error
    }
}

async fn fetch_opencc_dict(name: &str) -> Result<Dict, gloo_net::Error> {
    let text = Request::get(name).send().await?.text().await?;
    Ok(Dict::load_str(text))
}

pub async fn do_conversion_task(task: TaskRequest) -> Result<TaskResult, ConvertError> {
    // load simpecc dict
    let dict = match task.options.chinese_convertion.dict_name() {
        Some(name) => Some(fetch_opencc_dict(name).await?),
        None => None,
    };

    let check_file_size = |f: &File| {
        let size = f.size() as usize;
        if size > FILE_SIZE_LIMIT {
            Err(ConvertError::TooLarge {
                size,
                limit: FILE_SIZE_LIMIT,
            })
        } else {
            Ok(())
        }
    };

    let reader = FileReaderSync::new()?;
    let (content, meta, mime) = if task.files.len() <= 1
        && !task.files[0]
            .0
            .name()
            .to_ascii_lowercase()
            .ends_with(".zip")
    {
        // single file, no zip
        let input_buf = reader.read_to_vec(&task.files.first().ok_or(ConvertError::NoFile)?.0)?;
        let (output, meta) = convert_single_file(&input_buf, &task.options, &dict)?;
        (output, meta, MIME_SRT)
    } else {
        // check file size
        for FileWrap(file) in task.files.iter() {
            check_file_size(file)?;
        }
        // mulpitle files, with zip
        let mut meta = ConvertMeta::default();
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let zip_file_opt =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for result in FileWalk::new(task.files, reader) {
            let (mut path, buf) = result?;
            path.set_extension("srt");
            let (output, meta_) = convert_single_file(&buf, &task.options, &dict)?;
            meta += meta_;
            zip.start_file(path.to_string_lossy(), zip_file_opt)?;
            zip.write_all(&output).map_err(ZipError::Io)?;
        }
        let zip = zip.finish()?;
        (zip.into_inner().into_boxed_slice(), meta, MIME_ZIP)
    };
    // create blob url
    let file_blob = create_blob(&content, mime)?;
    let file_url = Url::create_object_url_with_blob(&file_blob)?;
    Ok(TaskResult { file_url, meta })
}

fn detect_encoding(input: &[u8]) -> Option<&'static Encoding> {
    let mut detector = EncodingDetector::new();
    for chunk in input.chunks(256) {
        if detector.feed(chunk, false) {
            let (encoding, sure) = detector.guess_assess(None, true);
            if sure {
                return Some(encoding);
            }
        }
    }
    detector.feed(&[], true);
    let (encoding, sure) = detector.guess_assess(None, true);
    Some(encoding).take_if(|_| sure)
}

fn convert_single_file(
    input: &[u8],
    opts: &Options,
    dict: &Option<Dict>,
) -> Result<(Box<[u8]>, ConvertMeta), ConvertError> {
    let mut meta = ConvertMeta::default();
    // set encodings
    let ass_charset = if opts.ass_charset.is_empty() {
        detect_encoding(input).ok_or(ConvertError::EncodingDetect)?
    } else {
        Encoding::for_label(opts.ass_charset.as_bytes())
            .ok_or(ConvertError::EncodingLabel(opts.ass_charset.clone()))?
    };
    let srt_charset = if opts.srt_charset.is_empty() {
        UTF_8
    } else {
        Encoding::for_label(opts.srt_charset.as_bytes())
            .ok_or(ConvertError::EncodingLabel(opts.srt_charset.clone()))?
    };
    if srt_charset == UTF_16BE || srt_charset == UTF_16LE {
        return Err(ConvertError::Utf16Output);
    }

    // set text map (for line strip & chinese convertion)
    let text_map = for<'a> |text: Cow<'a, str>| -> Cow<'a, str> {
        let text = opts.line_strip.strip(text);
        if let Some(dict) = dict {
            Cow::Owned(dict.replace_all(&text))
        } else {
            text
        }
    };

    // decode & convert
    let (ass, ass_charset, has_error) = ass_charset.decode(input);
    meta.input_encoding.insert(ass_charset.name().to_string());
    meta.decode_error = has_error;
    let offset_secs = (opts.offset_millis as f32) / 1000.0;
    let srt = subtitle::ass_to_srt(&ass, true, Some(text_map), offset_secs)?;

    // encode
    if srt_charset == UTF_8 {
        meta.output_encoding.insert(srt_charset.name().to_string());
        meta.encode_error = false;
        Ok((srt.into_bytes().into_boxed_slice(), meta))
    } else {
        let (output, srt_charset, has_error) = srt_charset.encode(&srt);
        meta.output_encoding.insert(srt_charset.name().to_string());
        meta.encode_error = has_error;
        Ok((output.into_owned().into_boxed_slice(), meta))
    }
}

fn create_blob<T: AsRef<[u8]>>(buf: T, mime: &str) -> Result<Blob, JsValue> {
    let blob_opts = BlobPropertyBag::new();
    blob_opts.set_type(mime);
    let blob_parts = Array::new();
    blob_parts.push(&Uint8Array::from(buf.as_ref()));
    Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_opts)
}
