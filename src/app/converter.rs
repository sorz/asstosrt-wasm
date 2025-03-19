use futures::{
    channel::oneshot::{Receiver, channel},
    lock::{Mutex, MutexGuard},
};
use send_wrapper::SendWrapper;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::{File, MessageEvent, Worker, WorkerOptions, WorkerType, window};

use crate::{ConvertMeta, FileWrap, Options, TaskRequest, WorkerMessage, worker::ConvertError};

use super::task::BlobUrl;

#[derive(Debug, Clone)]
pub(crate) struct Converter {
    inner: Arc<Mutex<Inner>>,
}

impl Converter {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()).into(),
        }
    }

    pub(crate) async fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().await
    }
}

#[derive(Debug)]
pub(crate) struct Inner {
    worker: SendWrapper<Worker>,
    ready: Option<Receiver<()>>,
}

impl Inner {
    fn get_uri() -> Option<String> {
        window()?
            .document()?
            .document_element()?
            .get_attribute("data-worker-uri")
    }

    fn new() -> Self {
        let uri = Self::get_uri().unwrap_or("./worker_loader.js".to_string());
        log::debug!("spawning worker from {}", uri);
        let opts = WorkerOptions::new();
        opts.set_type(WorkerType::Module);
        let worker: Worker = Worker::new_with_options(&uri, &opts).expect("failed to spawn worker");

        let (ready_tx, ready_rx) = channel::<()>();
        let worker_ = worker.clone();
        let on_message = Closure::once(move |ev: MessageEvent| {
            match serde_wasm_bindgen::from_value(ev.data()) {
                Ok(WorkerMessage::WorkerReady) => ready_tx.send(()).unwrap(),
                Ok(msg) => log::warn!("unexpected message {:?}", msg),
                Err(err) => log::error!("failed to parse message {:?}", err),
            }
            worker_.set_onmessage(None);
        });
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        Self {
            worker: SendWrapper::new(worker),
            ready: Some(ready_rx),
        }
    }

    pub(crate) async fn convert(
        &mut self,
        options: Options,
        files: Vec<File>,
    ) -> Result<(BlobUrl, ConvertMeta), ConvertError> {
        // wait for worker ready
        if let Some(ready) = self.ready.take() {
            log::debug!("convert: wait for worker ready");
            ready.await?;
        }
        log::debug!("convert: {:?} files", files.len());
        // setup event listener
        let (result_tx, result_rx) = channel();
        let worker = self.worker.clone().take();
        let on_message = Closure::once(move |ev: MessageEvent| {
            match serde_wasm_bindgen::from_value(ev.data()) {
                Ok(WorkerMessage::TaskDone(result)) => result_tx.send(result).unwrap(),
                Ok(msg) => log::warn!("unexpected message {:?}", msg),
                Err(err) => log::error!("failed to parse message {:?}", err),
            }
            worker.set_onmessage(None);
        });
        let worker = self.worker.clone().take();
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        // send request
        let request = TaskRequest {
            options,
            files: files.into_iter().map(FileWrap).collect(),
        };
        worker.post_message(&serde_wasm_bindgen::to_value(&request).unwrap())?;
        // wait response
        result_rx.await?.map(|r| (BlobUrl::new(r.file_url), r.meta))
    }
}
