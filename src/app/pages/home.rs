use leptos::prelude::*;
use reactive_stores::Store;

use crate::{Options, app::components::options_form::OptionsForm};

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let options = Store::new(Options::default());

    view! {
        <div class="container">
            <h1>
                <abbr title="Advanced SubStation Alpha">ASS</abbr>
                /
                <abbr title="SubStation Alpha">SSA</abbr>
                to
                <abbr title="SubRip">SRT</abbr>
                Subtitles Converter
            </h1>
            <p>
                Only support for those newer browsers.<br />You can
                <a href="https://lab.sorz.org/tools/asstosrt/">try this</a>
                if it does not work for you.
            </p>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <h1>"Uh oh! Something went wrong!"</h1>

                    <p>"Errors: "</p>
                    // Render a list of errors as strings - good for development purposes
                    <ul>
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                .collect_view()
                        }}
                    </ul>
                }
            }>

                <h2>Options</h2>

                <details>
                    <summary>Show encodings, lines, zip, etc.</summary>

                    <form id="options">
                        <OptionsForm options=options />
                    </form>
                </details>

                <h2>Drop your ASS/SSA Files</h2>

                <h2>Save SRT Files</h2>

                <footer>
                    <p>Your files keep on your device and would NOT be uploaded to anywhere.</p>
                    <p>
                        Powered by
                        <a href="https://www.rust-lang.org/" title="The Rust Programming Language">
                            Rust
                        </a>and a set of lovely open-source projects.
                    </p>
                    <p>
                        Source code is avaiable on
                        <a href="https://github.com/sorz/asstosrt-wasm">GitHub</a>.
                    </p>
                </footer>
            </ErrorBoundary>
        </div>
    }
}
