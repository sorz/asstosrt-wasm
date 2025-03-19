leptos_i18n::load_locales!();
use std::time::Duration;

use converter::Converter;
use i18n::I18nContextProvider;
use leptos::prelude::*;
use leptos_meta::*;
use strum::{AsRefStr, Display, EnumString};

use pages::home::Home;

mod components;
mod converter;
mod pages;
pub(crate) mod storage;
mod task;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    // Converter service worker
    if use_context::<Converter>().is_none() {
        provide_context(Converter::new());
    }
    // Dark/light theme
    let theme = RwSignal::new(Theme::default());
    if use_context::<RwSignal<Theme>>().is_none() {
        provide_context(theme);
    }
    // Workaround for theme transision, which is enabled only if loaded = true
    let (loaded, set_loaded) = signal(false);
    set_interval(move || *set_loaded.write() = true, Duration::from_secs(1));

    view! {
        <I18nContextProvider>
            <Html
                class:loaded=move || loaded.get()
                attr:data-theme=move || theme.read().to_string()
            />
            <Meta charset="UTF-8" />
            <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <Home />
        </I18nContextProvider>
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum Theme {
    Auto,
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        match storage::get_parse(storage::Key::Theme) {
            Some(theme) => theme,
            None => Theme::Auto,
        }
    }
}

impl Theme {
    pub(crate) fn switch_next(&mut self) {
        *self = match self {
            Self::Auto => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::Auto,
        };
    }
}
