use leptos::prelude::*;
use leptos_i18n::t_string;

use crate::app::{
    Theme,
    i18n::{Locale, use_i18n},
    storage,
};

#[component]
pub(crate) fn ToggleBar() -> impl IntoView {
    view! {
        <ul class="toggle-bar">
            <LocaleSwitches />
            <li class="sep">|</li>
            <ThemeSwitch />
        </ul>
    }
}

#[component]
fn ThemeSwitch() -> impl IntoView {
    let i18n = use_i18n();
    let theme: RwSignal<Theme> = use_context().expect("theme not found on context");
    view! {
        <li class="theme">
            <button
                type="button"
                title=move || match theme.get() {
                    Theme::Auto => t_string!(i18n, theme_auto),
                    Theme::Light => t_string!(i18n, theme_light),
                    Theme::Dark => t_string!(i18n, theme_dark),
                }
                on:click=move |_| theme.update(|t| t.switch_next())
            >
                {move || match theme.get() {
                    Theme::Auto => "🌗",
                    Theme::Light => "🌕",
                    Theme::Dark => "🌑",
                }}
            </button>
        </li>
    }
}

#[component]
fn LocaleSwitches() -> impl IntoView {
    let i18n = use_i18n();

    // read locale from localStorage or navigator.languages
    Effect::new(move |_| {
        let locale = match storage::get_parse(storage::Key::Locale) {
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
        if let Err(err) = storage::set(storage::Key::Locale, locale) {
            log::error!("failed to set locale in storage: {:?}", err);
        }
    };

    let all_locales = [Locale::en, Locale::zh_Hans, Locale::zh_Hant];
    move || {
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
    }
}
