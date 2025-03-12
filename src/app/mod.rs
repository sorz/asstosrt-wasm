leptos_i18n::load_locales!();
use converter::Converter;
use i18n::I18nContextProvider;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use pages::{home::Home, not_found::NotFound};

mod components;
mod converter;
mod pages;
mod task;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    if use_context::<Converter>().is_none() {
        provide_context(Converter::new());
    }
    view! {
        <I18nContextProvider>
            <Html />
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
