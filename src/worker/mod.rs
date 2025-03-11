mod subtitle;
mod zip;

use chardetng::EncodingDetector;
use encoding_rs::Encoding;
use gloo_net::http::Request;
use js_sys::{Array, Uint8Array};
use serde::Deserialize;
use simplecc::Dict;
use std::{borrow::Cow, io::Cursor};
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, FileReaderSync, Url};

use crate::{Options, TaskRequest, TaskResult};

const MIME_SRT: &str = "text/srt";
const MIME_ZIP: &str = "application/zip";

#[derive(Deserialize, Debug, Clone)]
enum Lines {
    First,
    Last,
    All,
}

type StrError = Cow<'static, str>;

async fn fetch_simplecc_dict(name: &str) -> Result<Dict, StrError> {
    let text = Request::get(name)
        .send()
        .await
        .map_err(|e| Cow::Owned(format!("failed to fetch dict: {:?}", e)))?
        .text()
        .await
        .map_err(|e| Cow::Owned(format!("failed to fetch dict: {:?}", e)))?;
    Ok(Dict::load_str(text))
}

pub async fn do_conversion_task(task: TaskRequest) -> Result<TaskResult, String> {
    // load simpecc dict
    let dict = match task.options.chinese_convertion.dict_name() {
        Some(name) => Some(fetch_simplecc_dict(name).await?),
        None => None,
    };

    let reader = FileReaderSync::new().unwrap();
    // TODO: check & limit input file size
    let (content, mime) = if task.files.len() <= 1 {
        // single file, no zip
        let input_buf = {
            let file = &task.files.first().unwrap().0;
            let array = reader.read_as_array_buffer(file).unwrap();
            let mut buf = vec![0u8; array.byte_length().try_into().unwrap()];
            Uint8Array::new(&array).copy_to(&mut buf);
            buf
        };
        let output = convert_single_file(&input_buf, &task.options, &dict)?;
        (output, MIME_SRT)
    } else {
        // mulpitle files, with zip
        let files = task.files.into_iter().map(|file| {
            let name = file.0.name().trim_end_matches(".ass").to_string() + ".srt";
            let content = reader
                .read_as_array_buffer(&file.0)
                .map(|array| Uint8Array::new(&array));
            (name, content)
        });
        let mut input_buf = vec![0u8; 0];
        let mut output_buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut output_buf);
        for (name, file) in files {
            let file = file.expect("failed to open file");
            input_buf.resize(file.length().try_into().unwrap(), 0);
            file.copy_to(&mut input_buf);
            let output = convert_single_file(&input_buf, &task.options, &dict)?;
            zip.write_file(&name, output.as_ref()).unwrap();
        }
        zip.close().unwrap();
        (output_buf.into_inner().into_boxed_slice(), MIME_ZIP)
    };
    // create blob url
    let file_blob = create_blob(&content, mime).unwrap();
    let file_url = Url::create_object_url_with_blob(&file_blob).unwrap();
    Ok(TaskResult { file_url })
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
) -> Result<Box<[u8]>, StrError> {
    // set encodings
    let ass_charset = if opts.ass_charset.is_empty() {
        detect_encoding(input).ok_or(Cow::Borrowed("failed to detect encoding"))?
    } else {
        Encoding::for_label(opts.ass_charset.as_bytes())
            .ok_or(Cow::Borrowed("unknown input encoding"))?
    };
    let srt_charset = if opts.srt_charset.is_empty() {
        ass_charset
    } else {
        Encoding::for_label(opts.srt_charset.as_bytes())
            .ok_or(Cow::Borrowed("unknown output encoding"))?
    };

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
    let (ass, _, has_error) = ass_charset.decode(input);
    if has_error && !opts.ignore_charset_error {
        return Err(Cow::Borrowed("decoding error"));
    }
    let srt = subtitle::ass_to_srt(&ass, true, Some(text_map), opts.offset_secs)?;

    // encode
    // TODO: insert BOM for utf-16
    let (output, _, has_error) = srt_charset.encode(&srt);
    if has_error && !opts.ignore_charset_error {
        return Err(Cow::Borrowed("decoding error"));
    }

    // TODO: remove clone for utf-8 output
    Ok(output.into_owned().into_boxed_slice())
}

fn create_blob<T: AsRef<[u8]>>(buf: T, mime: &str) -> Result<Blob, JsValue> {
    let blob_opts = BlobPropertyBag::new();
    blob_opts.set_type(mime);
    let blob_parts = Array::new();
    blob_parts.push(&Uint8Array::from(buf.as_ref()));
    Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_opts)
}
