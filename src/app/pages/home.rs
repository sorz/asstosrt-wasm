use leptos::prelude::*;
use leptos_i18n::{t, t_string};
use leptos_meta::Title;
use reactive_stores::Store;

use crate::{
    Options,
    app::{
        components::{FileInput, OptionsForm, TaskList, ToggleBar},
        converter::Converter,
        i18n::use_i18n,
        task::{Task, Tasks},
    },
};

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let i18n = use_i18n();
    let options = Store::new(Options::default());
    let (tasks, set_tasks) = signal(Tasks::default());
    let converter: Converter = use_context().expect("converter not found");

    let convert = Action::new_local(move |task: &Task| {
        let task = *task;
        let options = options.read_untracked().clone();
        let converter = converter.clone();
        async move {
            let mut conv = converter.lock().await;
            let files = task.set_working().expect("try to work on non-pending task");
            match conv.convert(options, files).await {
                Ok((file, meta)) => task.set_done(file, meta),
                Err(msg) => task.set_error(msg),
            }
        }
    });

    Effect::new(move |_| {
        let tasks = tasks.read();
        if !tasks.any_working_task() {
            // do it in serial
            if let Some(task) = tasks.get_next_pending() {
                convert.dispatch_local(task);
            }
        }
    });

    view! {
        <Title text=move || t_string!(i18n, html_title) />
        <div class="container">
            <ToggleBar />
            <h1>
                <abbr title="Advanced SubStation Alpha">ASS</abbr>
                {t!(i18n, title_to)}
                <abbr title="SubRip">SRT</abbr>
                {t!(i18n, title_converter)}
            </h1>
            <ErrorBoundary fallback=move |errors| {
                view! {
                    <h1>{t!(i18n, error_title)}</h1>
                    <p>{t!(i18n, error_list)}</p>
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
                <details class="options">
                    <summary>{t!(i18n, opt_title)}</summary>
                    <form>
                        <OptionsForm options=options />
                    </form>
                </details>
                <FileInput on_files=move |files| {
                    if options.read().no_zip {
                        for file in files {
                            set_tasks.write().add(Task::new(vec![file]));
                        }
                    } else {
                        set_tasks.write().add(Task::new(files))
                    }
                } />
                <TaskList tasks set_tasks />
            </ErrorBoundary>
            <footer class:hide=move || !tasks.read().is_empty()>
                <p>
                    {t!(
                        i18n, footer_browser_compat, alt=move || view! {
                    <a href="https://lab.sorz.org/tools/asstosrt/">
                        {t!(i18n, footer_alt)}
                    </a>
                }
                    )}
                </p>
                <p>{t!(i18n, footer_file_stay_local)}</p>
                <p>
                    {t!(
                        i18n, footer_source_code, github=|| view! { <a href="https://github.com/sorz/asstosrt-wasm">GitHub</a> }
                    )}
                </p>
            </footer>
        </div>
    }
}
