use std::{fmt::Display, mem, sync::Arc};

use leptos::prelude::*;
use strum::EnumIs;
use uuid::Uuid;
use web_sys::{File, Url};

use super::converter::ConvertedFile;
use crate::worker::ConvertError;

#[derive(Debug, Clone, Default)]
pub(crate) struct Tasks(pub(crate) Vec<Task>);

impl Tasks {
    pub(crate) fn add(&mut self, task: Task) {
        self.0.push(task);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Set is_removing flag for all done/error tasks
    /// Return the number of modified tasks
    pub(crate) fn clear_ended_prepare(&mut self) -> usize {
        let mut n = 0;
        for task in self.0.iter() {
            let state = task.state.read_untracked();
            if state.is_done() || state.is_error() {
                *task.is_removing.write() = true;
                n += 1
            }
        }
        n
    }

    // Clear all tasks which has set is_removing flag
    pub(crate) fn clear(&mut self) {
        self.retain(|task| !task.is_removing.get_untracked());
    }

    /// Check if any task is done or errored
    pub(crate) fn any_ended(&self) -> bool {
        self.0
            .iter()
            .any(|task| task.state.with(|state| state.is_done() || state.is_error()))
    }

    pub(crate) fn any_working_task(&self) -> bool {
        self.0.iter().any(|task| task.state.read().is_working())
    }

    pub(crate) fn get_next_pending(&self) -> Option<Task> {
        self.0
            .iter()
            .find(|task| task.state.read().is_pending())
            .copied()
    }

    fn retain(&mut self, mut f: impl FnMut(&Task) -> bool) {
        self.0.retain(|task| {
            let retain = f(task);
            if !retain {
                task.filenames.dispose();
                task.state.dispose();
                task.is_removing.dispose();
            }
            retain
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Task {
    pub(crate) id: Uuid,
    pub(crate) filenames: RwSignal<Vec<String>>,
    pub(crate) state: RwSignal<TaskState, LocalStorage>,
    /// To be removed from task list (waiting for animation end)
    pub(crate) is_removing: RwSignal<bool>,
}

#[derive(Debug, Clone, EnumIs)]
pub(crate) enum TaskState {
    Pending { files: Vec<File> },
    Working,
    Done(Arc<ConvertedFile>),
    Error(ConvertError),
}

impl Task {
    pub(crate) fn new(files: Vec<File>) -> Self {
        let filenames = files.iter().map(|f| f.name()).collect();
        Self {
            id: Uuid::new_v4(),
            filenames: RwSignal::new(filenames),
            state: RwSignal::new_local(TaskState::Pending { files }),
            is_removing: RwSignal::new(false),
        }
    }

    pub(crate) fn set_working(&self) -> Option<Vec<File>> {
        self.state
            .try_update(|state| match mem::replace(state, TaskState::Working) {
                TaskState::Pending { files } => Some(files),
                _ => None,
            })
            .flatten()
    }

    pub(crate) fn set_done(&self, file: ConvertedFile) {
        self.state.set(TaskState::Done(file.into()));
    }

    pub(crate) fn set_error(&self, error: ConvertError) {
        self.state.set(TaskState::Error(error))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BlobUrl(String);

impl Drop for BlobUrl {
    fn drop(&mut self) {
        log::debug!("revoking blob url {}", self.0);
        if let Err(err) = Url::revoke_object_url(&self.0) {
            log::warn!("failed to revoke blob url {:?}", err);
        }
    }
}

impl BlobUrl {
    pub(crate) fn new(url: String) -> Self {
        Self(url)
    }
}

impl Display for BlobUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
