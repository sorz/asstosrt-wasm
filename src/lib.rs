mod subtitle;
mod zip;

use chardet::charset2encoding;
use encoding::{
    label::encoding_from_whatwg_label,
    types::{DecoderTrap, EncoderTrap, EncodingRef},
};
use js_sys::{Array, Uint8Array};
use serde::Deserialize;
use simplecc::Dict;
use std::{borrow::Cow, io::Cursor};
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, console};

#[derive(Deserialize, Debug, Clone)]
struct Charset(String);

#[derive(Deserialize, Debug, Clone)]
enum Lines {
    First,
    Last,
    All,
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct IgnoreCodecErr(bool);

#[derive(Deserialize, Debug, Clone)]
struct Options {
    in_charset: Option<Charset>,
    out_charset: Option<Charset>,
    lines: Lines,
    ignore_codec_err: IgnoreCodecErr,
    conv_dict: Option<String>,
    offset_secs: f32,
}

type StrError = Cow<'static, str>;

impl TryFrom<Charset> for EncodingRef {
    type Error = StrError;
    fn try_from(value: Charset) -> Result<Self, Self::Error> {
        encoding_from_whatwg_label(&value.0).ok_or_else(|| "unknown charset name".into())
    }
}

impl From<IgnoreCodecErr> for EncoderTrap {
    fn from(val: IgnoreCodecErr) -> Self {
        if val.0 { Self::Replace } else { Self::Strict }
    }
}

impl From<IgnoreCodecErr> for DecoderTrap {
    fn from(val: IgnoreCodecErr) -> Self {
        if val.0 { Self::Replace } else { Self::Strict }
    }
}

fn detect_charset(mut s: &[u8]) -> Option<EncodingRef> {
    if s.len() > 4096 {
        s = &s[..4096];
    }
    let result = chardet::detect(s);
    console::log_1(&format!("chardet {:?}", result).into());
    encoding_from_whatwg_label(charset2encoding(&result.0))
}

fn convert(ass: Uint8Array, opts: Options) -> Result<Box<[u8]>, StrError> {
    let ass = ass.to_vec();
    let in_charset = if let Some(charset) = opts.in_charset {
        charset.try_into()?
    } else {
        detect_charset(&ass).ok_or(Cow::Borrowed("fail to detect ASS charset"))?
    };
    let out_charset = opts.out_charset.map_or(Ok(in_charset), |l| l.try_into())?;
    let dict: Option<Dict> = opts.conv_dict.map(|s| Dict::load_str(&s));
    let lines = opts.lines;
    let mapper = |s: String| {
        match lines {
            Lines::First => s.lines().next(),
            Lines::Last => s.lines().last(),
            Lines::All => Some(s.as_str()),
        }
        .map(|s| dict.as_ref().map_or(s.into(), |d| d.replace_all(s)))
    };

    let ass = in_charset.decode(&ass, opts.ignore_codec_err.into())?;
    let srt = subtitle::ass_to_srt(&ass, true, Some(mapper), opts.offset_secs)?;

    let mut output = Vec::new();
    // insert BOM for utf-16
    if out_charset
        .whatwg_name()
        .is_some_and(|n| n.starts_with("utf-16"))
    {
        out_charset.encode_to("\u{feff}", EncoderTrap::Strict, &mut output)?;
    }
    out_charset.encode_to(&srt, opts.ignore_codec_err.into(), &mut output)?;
    Ok(output.into_boxed_slice())
}

fn create_blob<T: AsRef<[u8]>>(buf: T, mime: &str) -> Result<Blob, JsValue> {
    let blob_opts = BlobPropertyBag::new();
    blob_opts.set_type(mime);
    let blob_parts = Array::new();
    blob_parts.push(&Uint8Array::from(buf.as_ref()));
    Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_opts)
}

#[wasm_bindgen(js_name = assToSrt)]
pub fn ass_to_srt(ass: Uint8Array, opts: JsValue) -> Result<Blob, JsValue> {
    let opts = serde_wasm_bindgen::from_value(opts).unwrap();
    let output = convert(ass, opts).map_err(|err| JsValue::from_str(&err))?;
    create_blob(output, "text/srt")
}

#[wasm_bindgen(js_name = assToSrtBulk)]
pub fn ass_to_srt_bulk(
    files: Vec<Uint8Array>,
    filenames: Vec<String>,
    opts: JsValue,
) -> Result<Blob, JsValue> {
    let opts: Options = serde_wasm_bindgen::from_value(opts).unwrap();
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let fname_srt = filenames
            .into_iter()
            .zip(files.into_iter().map(|f| convert(f, opts.clone())));
        for (fname, srt) in fname_srt {
            let srt = srt.map_err(|err| JsValue::from_str(&err))?;
            zip.write_file(&fname, srt.as_ref())
                .map_err(|_| JsValue::from_str("zip write error"))?;
        }
        zip.close()
            .map_err(|_| JsValue::from_str("zip close error"))?;
    }
    create_blob(buf.get_ref(), "application/zip")
}
