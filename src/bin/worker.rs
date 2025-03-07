use asstosrt_wasm::WorkerMessage;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::debug!("staring service worker");

    let scope: DedicatedWorkerGlobalScope = JsValue::from(js_sys::global()).into();
    let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |ev: MessageEvent| {
        web_sys::console::log_2(&"new message (worker)".into(), &ev.data());
        // TODO
    });
    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    let ready = serde_wasm_bindgen::to_value(&WorkerMessage::Ready).unwrap();
    scope.post_message(&ready).expect("failed to post message");
    on_message.forget();
}
