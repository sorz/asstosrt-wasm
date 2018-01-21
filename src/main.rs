#[macro_use]
extern crate stdweb;
extern crate encoding;
extern crate asstosrt_wasm;

use stdweb::{Value, UnsafeTypedArray};
use encoding::types::EncoderTrap;
use encoding::label::encoding_from_whatwg_label;


macro_rules! throw {
    ( $e:expr ) => {
        {
            js!( throw @{$e} );
            unreachable!()
        }
    };
}

fn ass_to_srt(ass: String, out_charset: String) -> Value {
    let out_charset = encoding_from_whatwg_label(&out_charset)
        .unwrap_or_else(|| throw!("invalid output charset name"));
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
