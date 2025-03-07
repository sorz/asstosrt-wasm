use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

thread_local!(static WORKER: RefCell<Worker> = panic!("worker not initlized on current thread"));

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Converter;

impl Converter {
    pub(crate) fn new() -> Self {
        log::debug!("spawning worker");
        let opts = WorkerOptions::new();
        opts.set_type(WorkerType::Module);
        let worker: Worker =
            Worker::new_with_options("./worker_loader.js", &opts).expect("failed to spawn worker");

        let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |ev: MessageEvent| {
            web_sys::console::log_2(&"new message (app)".into(), &ev.data());
            // TODO
        });
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        WORKER.set(worker);
        Self {}
    }
}
