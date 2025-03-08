use std::cell::LazyCell;

use leptos::prelude::*;
use reactive_stores::Store;
use web_sys::File;

use crate::{
    Options,
    app::{
        components::{FileInput, OptionsForm, TaskRow},
        converter::Converter,
        task::{Task, Tasks},
    },
};

const CONVERTER: LazyCell<Converter> = LazyCell::new(|| Converter::new());

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let options = Store::new(Options::default());
    let (tasks, set_tasks) = signal(Tasks::default());

    let convert = Action::new_local(move |files: &Vec<File>| {
        let files = files.clone();
        let options = options.read_untracked().clone();
        async move {
            CONVERTER.convert(options, files).await;
        }
    });

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
                <details>
                    <summary>Show options</summary>
                    <form id="options">
                        <OptionsForm options=options />
                    </form>
                </details>

                <FileInput on_files=move |files| set_tasks.write().add(Task::new(files)) />
                <ul class="task-list">
                    <For
                        each=move || tasks.get().0.into_iter().rev()
                        key=|task| task.id
                        children=move |task| {
                            view! { <TaskRow task /> }
                        }
                    />
                </ul>

            </ErrorBoundary>

            <p>
                Only new browsers are supported. If that dose not work,
                <a href="https://lab.sorz.org/tools/asstosrt/">try this</a>.
            </p>

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
        </div>
    }
}
