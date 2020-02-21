#[macro_use]
extern crate stdweb;
use chardet::charset2encoding;
use encoding::{
    label::encoding_from_whatwg_label,
    types::{DecoderTrap, EncoderTrap, EncodingRef},
};
use serde::Deserialize;
use simplecc::Dict;
use std::io::Cursor;
use stdweb::{web::ArrayBuffer, UnsafeTypedArray, Value};

use asstosrt_wasm::{subtitle, zip::ZipWriter};

macro_rules! throw {
    ( $e:expr ) => {
        {
            js!( throw @{$e} );
            unreachable!()
        }
    };
}

macro_rules! log {
    ( $e:expr ) => {
        {
            let output = $e;
            js!( console.log(@{output}) );
        }
    };
}

macro_rules! try_js {
    ( $e:expr ) => {
        $e.unwrap_or_else(|e| throw!(format!("{}", e)))
    };
    ( $e:expr, $m:expr, err) => {
        $e.unwrap_or_else(|e| throw!(format!("{}: {}", $m, e)))
    };
    ( $e:expr, $m:expr ) => {
        $e.unwrap_or_else(|| throw!(format!("{}", $m)))
    };
}

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
}
js_deserializable!(Options);

impl Into<EncodingRef> for Charset {
    fn into(self) -> EncodingRef {
        try_js!(encoding_from_whatwg_label(&self.0), "unknown charset name")
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
    log!(format!("chardet {:?}", result));
    encoding_from_whatwg_label(charset2encoding(&result.0))
}

fn convert(ass: ArrayBuffer, opts: Options) -> Box<[u8]> {
    let ass: Vec<u8> = ass.into();
    let in_charset = opts.in_charset.map_or_else(
        || try_js!(detect_charset(&ass), "fail to detect ASS charset"),
        |l| l.into(),
    );
    let out_charset = opts.out_charset.map_or(in_charset, |l| l.into());
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

    let ass = try_js!(
        in_charset.decode(&ass, opts.ignore_codec_err.into()),
        "fail to decode",
        err
    );
    let srt = try_js!(subtitle::ass_to_srt(&ass, true, Some(mapper)));

    let mut output = Vec::new();
    // insert BOM for utf-16
    if out_charset
        .whatwg_name()
        .map_or(false, |n| n.starts_with("utf-16"))
    {
        try_js!(out_charset.encode_to("\u{feff}", EncoderTrap::Strict, &mut output));
    }

    try_js!(
        out_charset.encode_to(&srt, opts.ignore_codec_err.into(), &mut output),
        "fail to encode",
        err
    );
    output.into_boxed_slice()
}

fn ass_to_srt(ass: ArrayBuffer, opts: Options) -> Value {
    let output = convert(ass, opts);
    let output = unsafe { UnsafeTypedArray::new(&output) };
    js! (return new Blob([@{output}], {type: "text/srt"}))
}

fn ass_to_srt_bulk(files: Vec<ArrayBuffer>, filenames: Vec<String>, opts: Options) -> Value {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        filenames
            .into_iter()
            .zip(files.into_iter().map(|f| convert(f, opts.clone())))
            .for_each(|(fname, f)| try_js!(zip.write_file(&fname, &f[..])));
        try_js!(zip.close());
    }
    let output = unsafe { UnsafeTypedArray::new(buf.get_ref()) };
    js! (return new Blob([@{output}], {type: "application/zip"}))
}

fn main() {
    stdweb::initialize();
    js! {
        Module.exports.assToSrt = @{ass_to_srt};
        Module.exports.assToSrtBulk = @{ass_to_srt_bulk};
    }
}
