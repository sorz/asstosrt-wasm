leptos_i18n::load_locales!();
use converter::Converter;
use i18n::I18nContextProvider;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use strum::{Display, EnumString};

use pages::{home::Home, not_found::NotFound};

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
    view! {
        <I18nContextProvider>
            <Html attr:data-theme=move || theme.read().to_string() />
            <Meta charset="UTF-8" />
            <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <Router>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=Home />
                </Routes>
            </Router>
        </I18nContextProvider>
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
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
