#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate encoding;
extern crate chardet;
extern crate asstosrt_wasm;

use stdweb::{Value, UnsafeTypedArray};
use stdweb::web::ArrayBuffer;
use encoding::types::{EncodingRef, EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use chardet::charset2encoding;

use asstosrt_wasm::subtitle;
use asstosrt_wasm::simplecc::Dict;


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

#[derive(Deserialize, Debug)]
struct Charset (String);

#[derive(Deserialize, Debug)]
enum Lines { First, Last, All }

#[derive(Deserialize, Debug)]
enum ChineseConv { None, S2T, T2S }

#[derive(Deserialize, Debug, Clone, Copy)]
struct IgnoreCodecErr (bool);

#[derive(Deserialize, Debug)]
struct Options {
    in_charset: Option<Charset>,
    out_charset: Option<Charset>,
    chinese_conv: ChineseConv,
    lines: Lines,
    ignore_codec_err: IgnoreCodecErr,
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

fn ass_to_srt(ass: ArrayBuffer, opts: Options) -> Value {
    let ass: Vec<u8> = ass.into();
    let in_charset = opts.in_charset.map_or_else(
        || try_js!(detect_charset(&ass), "fail to detect ASS charset"),
        |l| l.into());
    let out_charset = opts.out_charset.map_or(in_charset, |l| l.into());
    let dict = match opts.chinese_conv {
        ChineseConv::S2T => Some(Dict::default_s2t()),
        ChineseConv::T2S => Some(Dict::default_t2s()),
        ChineseConv::None => None,
    };
    let lines = opts.lines;
    let mapper = move |s: String| {
        match lines {
            Lines::First => s.lines().next(),
            Lines::Last => s.lines().last(),
            Lines::All => Some(s.as_str()),
        }.map(|s| dict.map_or(s.into(), |d| d.replace_all(s)))
    };

    let ass = try_js!(in_charset.decode(&ass,
            opts.ignore_codec_err.into()), "fail to decode", err);
    let srt = try_js!(subtitle::ass_to_srt(&ass, true, Some(mapper)));

    let mut output = Vec::new();
    // insert BOM for utf-16
    if out_charset.whatwg_name().map_or(false, |n| n.starts_with("utf-16")) {
        try_js!(out_charset.encode_to(
            "\u{feff}", EncoderTrap::Strict, &mut output));
    }

    try_js!(out_charset.encode_to(&srt,
        opts.ignore_codec_err.into(), &mut output), "fail to encode", err);
    let output = unsafe { UnsafeTypedArray::new(&output) };
    js! (return new Blob([@{output}], {type: "text/srt"}))
}

fn main() {
    stdweb::initialize();
    js! {
        Module.exports.assToSrt = @{ass_to_srt};
    }
}
