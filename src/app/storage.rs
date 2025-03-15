use std::str::FromStr;

use leptos::prelude::*;
use strum::AsRefStr;
use wasm_bindgen::JsValue;

#[derive(Debug, AsRefStr)]
#[strum(prefix = "ass2srt:")]
pub(crate) enum Key {
    Locale,
    Theme,
    Options,
}

fn local() -> Result<web_sys::Storage, JsValue> {
    window()
        .local_storage()?
        .ok_or(JsValue::from_str("null localStorage"))
}

pub(crate) fn set<V>(key: Key, value: V) -> Result<(), JsValue>
where
    V: AsRef<str>,
{
    local()?.set_item(key.as_ref(), value.as_ref())
}

pub(crate) fn get(key: Key) -> Result<Option<String>, JsValue> {
    local()?.get_item(key.as_ref())
}

pub(crate) fn get_parse<V: FromStr>(key: Key) -> Option<V> {
    get(key).ok().flatten().and_then(|s| s.parse().ok())
}
