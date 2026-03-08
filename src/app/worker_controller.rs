use std::{
    collections::VecDeque,
    time::{Instant, SystemTime},
};

pub use crate::contracts::sync::{
    SyncJob, SyncJobEvent, SyncJobKind, SyncSource, SyncStatusSnapshot,
};

mod control;
mod events;
mod queue;

pub struct WorkerController {
    queue: VecDeque<SyncJob>,
    active: Option<SyncJob>,
    error_count: u8,
    paused: bool,
    last_error: Option<String>,
    last_failed_job: Option<SyncJob>,
    last_pull: Option<SystemTime>,
    last_push: Option<SystemTime>,
}

impl WorkerController {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            active: None,
            error_count: 0,
            paused: false,
            last_error: None,
            last_failed_job: None,
            last_pull: None,
            last_push: None,
        }
    }
}

impl Default for WorkerController {
    fn default() -> Self {
        Self::new()
    }
}
