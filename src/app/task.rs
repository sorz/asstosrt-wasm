use std::{fmt::Display, mem, sync::Arc};

use leptos::prelude::*;
use strum::EnumIs;
use uuid::Uuid;
use web_sys::{File, Url};

use crate::worker::ConvertError;

#[derive(Debug, Clone, Default)]
pub(crate) struct Tasks(pub(crate) Vec<Task>);

impl Tasks {
    pub(crate) fn add(&mut self, task: Task) {
        self.0.push(task);
    }

    pub(crate) fn remove(&mut self, task_id: Uuid) {
        self.retain(|task| task.id != task_id);
    }

    /// Clear all done/error tasks
    pub(crate) fn clear_ended(&mut self) {
        self.retain(|task| {
            let state = task.state.read();
            state.is_pending() || state.is_working()
        });
    }

    /// Check if any task is done or errored
    pub(crate) fn any_ended(&self) -> bool {
        self.0
            .iter()
            .any(|task| task.state.with(|state| state.is_done() || state.is_error()))
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
                task.state.dispose();
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
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIs)]
pub(crate) enum TaskState {
    Pending { files: Vec<File> },
    Working,
    Done { file: Arc<BlobUrl> },
    Error { error: ConvertError },
}

impl Task {
    pub(crate) fn new(files: Vec<File>) -> Self {
        let filenames = files.iter().map(|f| f.name()).collect();
        Self {
            id: Uuid::new_v4(),
            filenames: RwSignal::new(filenames),
            state: RwSignal::new_local(TaskState::Pending { files }),
        }
    }

    pub(crate) fn set_working(&self) -> Option<Vec<File>> {
        let mut ret = None;
        self.state.update(|state| {
            ret = match mem::replace(state, TaskState::Working) {
                TaskState::Pending { files } => Some(files),
                _ => None,
            };
        });
        ret
    }

    pub(crate) fn set_done(&self, file: BlobUrl) {
        self.state.set(TaskState::Done { file: file.into() });
    }

    pub(crate) fn set_error(&self, error: ConvertError) {
        self.state.set(TaskState::Error { error })
    }

    pub(crate) fn output_filename(&self) -> String {
        let filenames = self.filenames.read();
        let name1 = filenames.first().and_then(|n| n.strip_suffix(".ass"));
        let name2 = filenames.last().and_then(|n| n.strip_suffix(".ass"));
        match (name1, name2) {
            // zip file
            (Some(name1), Some(name2)) if name1 != name2 => {
                let common: String = name1
                    .chars()
                    .zip(name2.chars())
                    .take_while(|(c1, c2)| c1 == c2)
                    .map(|(c, _)| c)
                    .collect();
                if common.is_empty() {
                    format!("ass2srt-{}.zip", self.id)
                } else {
                    format!("{}.zip", common)
                }
            }
            // srt file (w/o zip)
            (Some(name), _) => format!("{}.srt", name),
            // unreachable (if no bug)
            _ => format!("ass2srt-{}.srt", self.id),
        }
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
