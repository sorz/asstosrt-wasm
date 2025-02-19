mod subtitle;
mod zip;

use chardet::charset2encoding;
use encoding::{
    label::encoding_from_whatwg_label,
    types::{DecoderTrap, EncoderTrap, EncodingRef},
};
use js_sys::Uint8Array;
use serde::Deserialize;
use simplecc::Dict;
use std::{borrow::Cow, io::Cursor};
use wasm_bindgen::prelude::*;
use web_sys::{console, Blob, BlobPropertyBag};

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

impl Into<EncoderTrap> for IgnoreCodecErr {
    fn into(self) -> EncoderTrap {
        if self.0 {
            EncoderTrap::Replace
        } else {
            EncoderTrap::Strict
        }
    }
}

impl Into<DecoderTrap> for IgnoreCodecErr {
    fn into(self) -> DecoderTrap {
        if self.0 {
            DecoderTrap::Replace
        } else {
            DecoderTrap::Strict
        }
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
        detect_charset(&ass).ok_or_else(|| Cow::Borrowed("fail to detect ASS charset"))?
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
        .map_or(false, |n| n.starts_with("utf-16"))
    {
        out_charset.encode_to("\u{feff}", EncoderTrap::Strict, &mut output)?;
    }
    out_charset.encode_to(&srt, opts.ignore_codec_err.into(), &mut output)?;
    Ok(output.into_boxed_slice())
}

#[wasm_bindgen]
pub fn ass_to_srt(ass: Uint8Array, opts: JsValue) -> Result<Blob, JsValue> {
    let opts = serde_wasm_bindgen::from_value(opts).unwrap();
    let output: Uint8Array = convert(ass, opts)
        .map_err(|err| JsValue::from_str(&err))?
        .as_ref()
        .into();
    let blob_opts = BlobPropertyBag::new();
    blob_opts.set_type("text/srt");
    Blob::new_with_u8_array_sequence_and_options(&output, &blob_opts)
}

#[wasm_bindgen]
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

    let output: Uint8Array = buf.get_ref().as_slice().into();
    let blob_opts = BlobPropertyBag::new();
    blob_opts.set_type("application/zip");
    Blob::new_with_u8_array_sequence_and_options(&output, &blob_opts)
}
