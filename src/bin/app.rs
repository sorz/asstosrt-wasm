use leptos::prelude::*;

use asstosrt_wasm::app::App;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    mount_to_body(|| {
        view! { <App /> }
    })
}
