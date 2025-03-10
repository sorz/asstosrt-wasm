use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

use asstosrt_wasm::{TaskRequest, WorkerMessage, worker::do_conversion_task};

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::debug!("staring service worker");

    let scope: DedicatedWorkerGlobalScope = JsValue::from(js_sys::global()).into();
    let scope_ = scope.clone();
    let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |ev: MessageEvent| {
        let request: TaskRequest = match serde_wasm_bindgen::from_value(ev.data()) {
            Ok(req) => req,
            Err(err) => {
                log::error!("unexpected message {:?}", err);
                return;
            }
        };
        let scope = scope_.clone();
        spawn_local(async move {
            let result = do_conversion_task(request).await;
            let result = serde_wasm_bindgen::to_value(&WorkerMessage::TaskDone(result)).unwrap();
            scope.post_message(&result).unwrap();
        });
    });
    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    let ready = serde_wasm_bindgen::to_value(&WorkerMessage::WorkerReady).unwrap();
    scope.post_message(&ready).expect("failed to post message");
    on_message.forget();
}
