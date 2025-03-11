use leptos::prelude::*;
use reactive_stores::Store;

use crate::{
    Options,
    app::{
        components::{FileInput, OptionsForm, TaskList},
        converter::Converter,
        task::{Task, Tasks},
    },
};

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let options = Store::new(Options::default());
    let (tasks, set_tasks) = signal(Tasks::default());
    let converter: Converter = use_context().expect("converter not found");

    let convert = Action::new_local(move |task: &Task| {
        let task = *task;
        let files = task.set_working().unwrap();
        let options = options.read_untracked().clone();
        let converter = converter.clone();
        async move {
            match converter.convert(options, files).await {
                Ok(file) => task.set_done(file),
                Err(msg) => task.set_error(msg),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(task) = tasks.get().get_next_pending() {
            convert.dispatch_local(task);
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
                <TaskList tasks set_tasks />

            </ErrorBoundary>

            <footer>
                <p>
                    Only new browsers are supported. If that dose not work,
                    <a href="https://lab.sorz.org/tools/asstosrt/">try this</a>.
                </p>
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
