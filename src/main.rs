mod app;
mod drag;
mod form;

use crate::app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
