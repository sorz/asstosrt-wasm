#[macro_use]
extern crate stdweb;
extern crate encoding;
extern crate asstosrt_wasm;

use stdweb::{Value, UnsafeTypedArray};
use stdweb::web::ArrayBuffer;
use encoding::all::UTF_8;
use encoding::types::{EncodingRef, EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;


macro_rules! throw {
    ( $e:expr ) => {
        {
            js!( throw @{$e} );
            unreachable!()
        }
    };
}

fn ass_to_srt(ass: ArrayBuffer,
              in_charset: Option<String>,
              out_charset: Option<String>,
        ) -> Value {
    let in_charset = in_charset.map_or(UTF_8 as EncodingRef,
        |l| encoding_from_whatwg_label(&l)
            .unwrap_or_else(|| throw!("invalid ASS charset name")));
    let out_charset = out_charset.map_or(UTF_8 as EncodingRef,
        |l| encoding_from_whatwg_label(&l)
            .unwrap_or_else(|| throw!("invalid SRT charset name")));

    let ass = in_charset.decode(&ass.into(), DecoderTrap::Replace)
        .unwrap_or_else(|e| throw!(format!("fail to decode: {}", e)));
    let srt = match asstosrt_wasm::ass_to_srt(&ass, true) {
        Ok(s) => s,
        Err(e) => throw!(e),
    };
    let srt = out_charset.encode(&srt, EncoderTrap::Replace)
        .unwrap_or_else(|e| throw!(format!("fail to encode: {}", e)));
    let srt = unsafe { UnsafeTypedArray::new(&srt) };
    js! (return new Blob([@{srt}], {type: "text/srt"}))
}

fn main() {
    stdweb::initialize();
    js! {
        Module.exports.assToSrt = @{ass_to_srt};
    }
}
