use leptos::prelude::*;
use wasm_bindgen::JsValue;

use crate::app::i18n::{Locale, use_i18n};

const STORAGE_KEY_LOCALE: &str = "ass2srt-locale";

fn local_storage() -> Result<web_sys::Storage, JsValue> {
    window()
        .local_storage()?
        .ok_or(JsValue::from_str("null localStorage"))
}

fn set_storage_item<K, V>(key: K, value: V) -> Result<(), JsValue>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    local_storage()?.set_item(key.as_ref(), value.as_ref())
}

fn get_storage_item<K>(key: K) -> Result<Option<String>, JsValue>
where
    K: AsRef<str>,
{
    local_storage()?.get_item(key.as_ref())
}

#[component]
pub(crate) fn ToggleBar() -> impl IntoView {
    let i18n = use_i18n();

    // read locale from localStorage or navigator.languages
    Effect::new(move |_| {
        let locale = match get_storage_item(STORAGE_KEY_LOCALE)
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
        {
            Some(locale) => locale,
            _ => match window().navigator().language().as_deref() {
                Some("zh-CN") | Some("zh-SG") | Some("zh") => Locale::zh_Hans,
                Some("zh-TW") | Some("zh-HK") | Some("zh-MO") => Locale::zh_Hant,
                _ => Locale::en,
            },
        };
        i18n.set_locale(locale);
    });

    let set_locale = move |locale: Locale| {
        i18n.set_locale(locale);
        if let Err(err) = set_storage_item(STORAGE_KEY_LOCALE, locale) {
            log::error!("failed to set locale in storage: {:?}", err);
        }
    };

    let all_locales = [Locale::en, Locale::zh_Hans, Locale::zh_Hant];
    let locale_switches = move || {
        all_locales
            .into_iter()
            .map(|locale| {
                view! {
                    <li class="locale">
                        <button
                            type="button"
                            prop:disabled=move || i18n.get_locale() == locale
                            on:click=move |_| set_locale(locale)
                            prop:title=move || match locale {
                                Locale::en => "English",
                                Locale::zh_Hans => "简体中文",
                                Locale::zh_Hant => "繁體中文",
                            }
                        >
                            {move || match locale {
                                Locale::en => "EN",
                                Locale::zh_Hans => "简",
                                Locale::zh_Hant => "繁",
                            }}
                        </button>
                    </li>
                }
            })
            .collect::<Vec<_>>()
    };

    view! { <ul class="toggle-bar">{locale_switches}</ul> }
}
