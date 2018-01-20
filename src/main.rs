#[macro_use]
extern crate stdweb;
extern crate asstosrt_wasm;

fn ass_to_srt(ass: String) -> String {
    match asstosrt_wasm::ass_to_srt(&ass, true) {
        Ok(s) => s,
        Err(e) => { js!( throw @{e}; ); unreachable!() }
    }
}

fn main() {
    stdweb::initialize();
    js! {
        Module.exports.assToSrt = @{ass_to_srt};
    }
}
