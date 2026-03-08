use std::time::{Instant, SystemTime};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncJobKind {
    Pull,
    Push,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncSource {
    Manual,
    Button,
    Startup,
    Interval,
}

#[derive(Clone, Debug)]
pub struct SyncJob {
    pub kind: SyncJobKind,
    pub source: SyncSource,
    pub created_at: SystemTime,
    pub retries: u8,
    pub next_attempt_at: Option<Instant>,
}

impl SyncJob {
    pub fn new(kind: SyncJobKind, source: SyncSource) -> Self {
        Self {
            kind,
            source,
            created_at: SystemTime::now(),
            retries: 0,
            next_attempt_at: None,
        }
    }
}

pub enum SyncJobEvent {
    Completed(SyncJob),
    Failed { job: SyncJob, error: String },
}

#[derive(Clone, Debug, Default)]
pub struct SyncStatusSnapshot {
    pub queue_len: usize,
    pub active: Option<SyncJob>,
    pub queue_entries: Vec<SyncJob>,
    pub last_pull: Option<SystemTime>,
    pub last_push: Option<SystemTime>,
    pub paused: bool,
    pub error_count: u8,
    pub last_error: Option<String>,
}
