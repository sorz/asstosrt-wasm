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
            <span class="state">
                {move || match *task.state.read() {
                    TaskState::Pending { .. } => "PENDING",
                    TaskState::Working => "WORKING",
                    TaskState::Done { .. } => "READY",
                    TaskState::Error { .. } => "ERROR",
                }}
            </span>
            <div class="title-line">{title}</div>
            {move || more_files().map(|m| view! { <div class="more-files">{m}</div> })}
        </li>
    }
}
