use std::mem;

use leptos::prelude::*;
use strum::EnumIs;
use uuid::Uuid;
use web_sys::{Blob, File};

#[derive(Debug, Clone, Default)]
pub(crate) struct Tasks(pub(crate) Vec<Task>);

impl Tasks {
    pub(crate) fn add(&mut self, task: Task) {
        self.0.push(task);
    }

    pub(crate) fn remove(&mut self, task_id: Uuid) {
        self.retain(|task| task.id == task_id);
    }

    /// Clear all done/error tasks
    pub(crate) fn clear_ended(&mut self) {
        self.retain(|task| {
            let state = task.state.read();
            state.is_done() || state.is_error()
        });
    }

    fn retain(&mut self, mut f: impl FnMut(&Task) -> bool) {
        self.0.retain(|task| {
            let retain = f(task);
            if !retain {
                task.state.dispose();
            }
            retain
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Task {
    pub(crate) id: Uuid,
    pub(crate) filenames: Vec<String>,
    pub(crate) state: RwSignal<TaskState, LocalStorage>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIs)]
pub(crate) enum TaskState {
    Pending { files: Vec<File> },
    Working,
    Done { file: Blob },
    Error { message: String },
}

impl Task {
    pub(crate) fn new(files: Vec<File>) -> Self {
        Self {
            id: Uuid::new_v4(),
            filenames: files.iter().map(|f| f.name()).collect(),
            state: RwSignal::new_local(TaskState::Pending { files }),
        }
    }

    pub(crate) fn set_working(&mut self) -> Option<Vec<File>> {
        let mut ret = None;
        self.state.update(|state| {
            ret = match mem::replace(state, TaskState::Working) {
                TaskState::Pending { files } => Some(files),
                _ => None,
            };
        });
        ret
    }

    pub(crate) fn set_done(&mut self, file: Blob) {
        self.state.set(TaskState::Done { file });
    }

    pub(crate) fn set_error(&mut self, message: String) {
        self.state.set(TaskState::Error { message })
    }
}
