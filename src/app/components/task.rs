use leptos::prelude::*;

use crate::app::task::{Task, Tasks};

#[component]
pub(crate) fn TaskRow(task: Task, set_tasks: WriteSignal<Tasks>) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);

    let title = move || {
        let fns = task.filenames.get();
        let n = fns.len();
        let title = fns.into_iter().next().expect("empty file list");
        let and_n_files = Some(n).take_if(|n| *n > 1).map(|n| {
            view! {
                <span>+{n - 1}</span>
                <button on:click=move |_| {
                    set_expanded.update(|c| *c = !*c)
                }>{if expanded() { "-" } else { "+" }}</button>
            }
        });
        view! { <h4>{title}{and_n_files}</h4> }
    };
    let sub_file_list = move || {
        Some(task.filenames.get())
            .take_if(|fns| fns.len() > 1)
            .into_iter()
            .flatten()
            .map(|f| view! { <li>{f}</li> })
            .collect::<Vec<_>>()
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
            {title}
            <ul class="files" class:show=move || expanded.get()>
                {sub_file_list}
            </ul>
            {move || Some(remove_button).take_if(|_| !task.state.read().is_working())}

        </li>
    }
}
