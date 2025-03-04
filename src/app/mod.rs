use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use pages::home::Home;

mod components;
mod pages;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

        <Title text="Welcome to Leptos CSR" />

        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router>
            <Routes fallback=|| view! { NotFound }>
                <Route path=path!("/") view=Home />
            </Routes>
        </Router>
    }
}
