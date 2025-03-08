use leptos::prelude::*;

use crate::app::task::{Task, TaskState};

#[component]
pub(crate) fn TaskRow(task: Task) -> impl IntoView {
    view! {
        <li
            class:pending=move || task.state.get().is_pending()
            class:working=move || task.state.get().is_working()
            class:done=move || task.state.get().is_done()
            class:error=move || task.state.get().is_error()
        >
            <h4>// TODO
            </h4>
        </li>
    }
}
