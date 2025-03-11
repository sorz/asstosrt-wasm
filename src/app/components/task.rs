use leptos::prelude::*;

use crate::app::task::{Task, TaskState, Tasks};

#[component]
pub(crate) fn TaskList(tasks: ReadSignal<Tasks>, set_tasks: WriteSignal<Tasks>) -> impl IntoView {
    view! {
        <ul class="task-list">
            <For
                each=move || tasks.get().0.into_iter().rev()
                key=|task| task.id
                children=move |task| {
                    view! { <TaskItem task set_tasks /> }
                }
            />
        </ul>
    }
}

#[component]
fn TaskItem(task: Task, set_tasks: WriteSignal<Tasks>) -> impl IntoView {
    let state_label = move || {
        view! {
            <span class="state">
                {move || match *task.state.read() {
                    TaskState::Pending { .. } => "⌛PENDING",
                    TaskState::Working => "🟢WORKING",
                    TaskState::Done { .. } => "🎉READY",
                    TaskState::Error { .. } => "⚠️ERROR",
                }}
            </span>
        }
    };
    let title = move || {
        let fns = task.filenames.get();
        let title = fns.into_iter().next().expect("empty file list");
        view! {
            <span class="title" prop:title=title>
                {title.clone()}
            </span>
        }
    };
    let more_files = move || {
        let n = task.filenames.read().len();
        Some(view! {
            <details>
                <summary>Total {n}files</summary>
                <ol>
                    <For
                        each=move || task.filenames.get().into_iter()
                        key=|f| f.clone()
                        children=move |f| view! { <li title=f>{f.clone()}</li> }
                    />
                </ol>
            </details>
        })
        .take_if(|_| n > 1)
    };
    let download_link = move || {
        view! {
            <a
                class="download"
                href=move || match task.state.get() {
                    TaskState::Done { file } => Some(file.to_string()),
                    _ => None,
                }
                prop:download=move || task.output_filename()
                prop:title=move || task.output_filename()
            >
                "💾"
            </a>
        }
    };

    let remove_button = move || {
        view! { <button on:click=move |_| set_tasks.write().remove(task.id)>X</button> }
    };

    view! {
        <li
            class:pending=move || task.state.read().is_pending()
            class:working=move || task.state.read().is_working()
            class:done=move || task.state.read().is_done()
            class:error=move || task.state.read().is_error()
        >
            <div class="columns">
                <div class="state-and-title">{state_label}{title}</div>
                {move || Some(download_link).take_if(|_| task.state.read().is_done())}
            </div>
            {move || more_files().map(|m| view! { <div class="more-files">{m}</div> })}
        </li>
    }
}
