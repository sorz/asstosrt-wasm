use std::collections::HashSet;

use leptos::prelude::*;
use leptos_i18n::{t, t_string};

use crate::{
    app::{
        components::DonateBanner,
        i18n::use_i18n,
        task::{Task, TaskState, Tasks},
    },
    worker::{ConvertError, FormatError},
};

#[component]
pub(crate) fn TaskList(tasks: ReadSignal<Tasks>, set_tasks: WriteSignal<Tasks>) -> impl IntoView {
    let i18n = use_i18n();
    // Set true when user download any srt file
    let (has_downloaded, set_has_downloaded) = signal(false);
    // Count how many task is playing remove animation
    let (remove_pending_count, set_remove_pending_count) = signal(0);
    // When all animation end, we clear the task list
    Effect::new(move |_| {
        if remove_pending_count.get() == 0 {
            set_tasks.write().clear();
        }
    });

    let clear_button = move || {
        view! {
            <button
                class="clear"
                on:click=move |_| {
                    *set_remove_pending_count.write() = set_tasks.write().clear_ended_prepare();
                }
            >
                {t!(i18n, task_action_clear)}
            </button>
        }
    };
    view! {
        <ul class="task-list">
            <For
                each=move || tasks.get().0.into_iter().rev()
                key=|task| task.id
                children=move |task| {
                    view! { <TaskItem task set_remove_pending_count set_has_downloaded /> }
                }
            />
            <Show when=move || has_downloaded()>
                <DonateBanner />
            </Show>
            {move || {
                Some(move || view! { <li class="actions">{clear_button}</li> })
                    .take_if(|_| tasks.get().any_ended())
            }}
        </ul>
    }
}

#[component]
fn TaskItem(
    task: Task,
    set_remove_pending_count: WriteSignal<usize>,
    set_has_downloaded: WriteSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();

    let state_label = move || {
        let (icon, label) = match *task.state.read() {
            TaskState::Pending { .. } => ("⌛", t_string!(i18n, task_state_pending)),
            TaskState::Working => ("🟢", t_string!(i18n, task_state_working)),
            TaskState::Done { .. } => ("🎉", t_string!(i18n, task_state_done)),
            TaskState::Error(_) => ("⚠️", t_string!(i18n, task_state_error)),
        };
        view! { <span class="state">{icon}{" "}{label}</span> }
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
                <summary>{t!(i18n, task_file_list_summary, n)}</summary>
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

    let error_message = move || match task.state.get() {
        TaskState::Error(err) => view! {
            <p class="error">
                "😢"
                {match err {
                    ConvertError::NoFile => unreachable!("UI must not pass empty list"),
                    ConvertError::TooLarge { size, limit } => {
                        t!(i18n, error_file_limit, size, limit).into_any()
                    }
                    ConvertError::FetchDict(msg) => t!(i18n, error_fetch_dict, msg).into_any(),
                    ConvertError::EncodingLabel(label) => {
                        t!(i18n, error_encoding_label, label).into_any()
                    }
                    ConvertError::EncodingDetect => t!(i18n, error_encoding_detect).into_any(),
                    ConvertError::Utf16Output => t!(i18n, error_utf16_output).into_any(),
                    ConvertError::Canceled => t!(i18n, error_canceled).into_any(),
                    ConvertError::Zip(msg) => t!(i18n, error_zip, msg).into_any(),
                    ConvertError::JsError { name, msg } => {
                        t!(i18n, error_js_error, name, msg).into_any()
                    }
                    ConvertError::Format(FormatError::NoFormatLine) => {
                        t!(i18n, error_format_no_format_line).into_any()
                    }
                    ConvertError::Format(FormatError::NoFormatLineField(field)) => {
                        t!(i18n, error_format_no_format_line_field, field=field.to_string())
                            .into_any()
                    }
                    ConvertError::Format(FormatError::NoField(field)) => {
                        t!(i18n, error_format_no_field, field=field.to_string()).into_any()
                    }
                    ConvertError::Format(FormatError::Time(string)) => {
                        t!(i18n, error_format_time, string).into_any()
                    }
                }}
            </p>
        }
        .into_any(),
        TaskState::Done(file) if file.meta.has_error() => {
            let meta_ = file.meta.clone();
            let input = move || display_encodings(meta_.input_encoding.clone());
            let meta_ = file.meta.clone();
            let output = move || display_encodings(meta_.output_encoding.clone());
            view! {
                <p class="error">
                    "⚠️"
                    {match (file.meta.decode_error, file.meta.encode_error) {
                        (true, false) => t!(i18n, warning_decoding, input).into_any(),
                        (false, true) => t!(i18n, warning_encoding, output).into_any(),
                        (true, true) => {
                            t!(i18n, warning_decoding_encoding, input, output).into_any()
                        }
                        (false, false) => ().into_any(),
                    }}
                </p>
            }
            .into_any()
        }
        TaskState::Done { .. } | TaskState::Pending { .. } | TaskState::Working => ().into_any(),
    };

    let download_link = move || match task.state.get() {
        TaskState::Done(file) => Some(view! {
            <a
                class="download"
                href=file.url.to_string()
                prop:download=file.name.clone()
                prop:title=file.name.clone()
                on:click=move |_| set_has_downloaded(true)
            >
                "💾"
            </a>
        }),
        _ => None,
    };
    view! {
        <li
            class="task"
            class:pending=move || task.state.read().is_pending()
            class:working=move || task.state.read().is_working()
            class:done=move || task.state.read().is_done()
            class:error=move || task.state.read().is_error()
            class:removing=move || task.is_removing.get()
            on:animationend=move |ev| {
                if ev.animation_name() == "fade-out" {
                    *set_remove_pending_count.write() -= 1;
                }
            }
        >
            <div class="columns">
                <div class="state-and-title">{state_label}{title}</div>
                {move || Some(download_link).take_if(|_| task.state.read().is_done())}
            </div>
            {move || more_files().map(|m| view! { <div class="more-files">{m}</div> })}
            {error_message}
        </li>
    }
}

fn display_encodings(encodings: HashSet<String>) -> String {
    match encodings.len() {
        0 => "[]".to_string(),
        1 => encodings.into_iter().next().unwrap(),
        _ => format!("[{}]", encodings.into_iter().collect::<Vec<_>>().join(", ")),
    }
}
