#[macro_use]
extern crate stdweb;
extern crate encoding;
extern crate chardet;
extern crate asstosrt_wasm;

use stdweb::{Value, UnsafeTypedArray};
use stdweb::web::ArrayBuffer;
use encoding::types::{EncodingRef, EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use chardet::charset2encoding;

use asstosrt_wasm::subtitle;


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


fn detect_charset(mut s: &[u8]) -> Option<EncodingRef> {
    if s.len() > 4096 {
        s = &s[..4096];
    }
    let result = chardet::detect(s);
    log!(format!("chardet {:?}", result));
    encoding_from_whatwg_label(charset2encoding(&result.0))
}

fn ass_to_srt(ass: ArrayBuffer,
              in_charset: Option<String>,
              out_charset: Option<String>,
        ) -> Value {
    let ass: Vec<u8> = ass.into();
    let in_charset = in_charset.map_or_else(
        || detect_charset(&ass)
            .unwrap_or_else(|| throw!("fail to detect ASS charset")),
        |l| encoding_from_whatwg_label(&l)
            .unwrap_or_else(|| throw!("invalid ASS charset name")));
    let out_charset = out_charset.map_or(in_charset,
        |l| encoding_from_whatwg_label(&l)
            .unwrap_or_else(|| throw!("invalid SRT charset name")));

    let ass = in_charset.decode(&ass, DecoderTrap::Replace)
        .unwrap_or_else(|e| throw!(format!("fail to decode: {}", e)));
    let srt = match subtitle::ass_to_srt(&ass, true) {
        Ok(s) => s,
        Err(e) => throw!(e),
    };

    let mut output = Vec::new();
    // insert BOM for utf-16
    if out_charset.whatwg_name().map_or(false, |n| n.starts_with("utf-16")) {
        out_charset.encode_to("\u{feff}", EncoderTrap::Strict, &mut output)
            .unwrap_or_else(|e| log!(format!("fail to insert BOM: {}", e)));
    }

    out_charset.encode_to(&srt, EncoderTrap::Replace, &mut output)
        .unwrap_or_else(|e| throw!(format!("fail to encode: {}", e)));
    let output = unsafe { UnsafeTypedArray::new(&output) };
    js! (return new Blob([@{output}], {type: "text/srt"}))
}

fn main() {
    stdweb::initialize();
    js! {
        Module.exports.assToSrt = @{ass_to_srt};
    }
}
