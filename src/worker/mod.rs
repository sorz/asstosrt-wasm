//mod zip;

use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DedicatedWorkerGlobalScope, File, MessageEvent};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    AddFile {
        #[serde(skip)]
        file: Option<File>,
    },
    PreloadDict(String),
}

impl Into<JsValue> for Command {
    fn into(self) -> JsValue {
        let array = Array::new();
        let json = JsValue::from_serde(&self).expect("failed to encoded as json");
        array.push(&json);
        if let Command::AddFile { file: Some(file) } = self {
            array.push(&file);
        }
        array.into()
    }
}

impl Into<Command> for JsValue {
    fn into(self) -> Command {
        let array = Array::from(&self);
        let mut cmd: Command = array
            .shift()
            .into_serde()
            .expect("failed to deocded as json");
        if let Command::AddFile { ref mut file } = cmd {
            file.replace(array.shift().unchecked_into());
        }
        cmd
    }
}

pub struct Worker {
    scope: DedicatedWorkerGlobalScope,
}

impl Worker {
    pub fn init() {
        let scope = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()));
        let worker = Self {
            scope: scope.clone(),
        };
        let callback = Closure::wrap(Box::new(move |msg: MessageEvent| {
            let cmd: Command = msg.data().into();
            worker.on_command(cmd);
        }) as Box<dyn Fn(MessageEvent)>);
        scope.set_onmessage(Some(&callback.into_js_value().into()));
    }

    fn on_command(&self, cmd: Command) {
        log::debug!("on_command: {:?}", cmd);
    }
}
