use std::{
    collections::VecDeque,
    time::{Instant, SystemTime},
};

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

impl Default for WorkerController {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn enqueue(&mut self, job: SyncJob) {
        if self.is_duplicate(job.kind) {
            return;
        }
        self.queue.push_back(job);
    }

    pub fn enqueue_front(&mut self, job: SyncJob) {
        self.queue.retain(|queued| queued.kind != job.kind);
        self.queue.push_front(job);
    }

    pub fn start_next(&mut self) -> Option<SyncJob> {
        if self.paused || self.active.is_some() {
            return None;
        }
        let now = Instant::now();
        let pos = self
            .queue
            .iter()
            .position(|job| job.next_attempt_at.is_none_or(|at| at <= now))?;
        let job = if pos == 0 {
            self.queue.pop_front()?
        } else {
            self.queue.remove(pos)?
        };
        self.active = Some(job.clone());
        Some(job)
    }

    pub fn handle_worker_event(&mut self, event: SyncJobEvent) {
        match event {
            SyncJobEvent::Completed(job) => {
                self.active = None;
                self.error_count = 0;
                self.last_error = None;
                self.last_failed_job = None;
                let now = SystemTime::now();
                match job.kind {
                    SyncJobKind::Pull => self.last_pull = Some(now),
                    SyncJobKind::Push => self.last_push = Some(now),
                }
            }
            SyncJobEvent::Failed { mut job, error } => {
                self.active = None;
                self.error_count = self.error_count.saturating_add(1);
                job.retries = job.retries.saturating_add(1);
                self.last_error = Some(error);

                if job.retries >= 3 || self.error_count >= 3 {
                    self.paused = true;
                    self.last_failed_job = Some(job);
                } else {
                    let backoff = backoff_for(job.retries);
                    job.next_attempt_at = Some(Instant::now() + backoff);
                    self.queue.push_back(job);
                }
            }
        }
    }

    pub fn pause(&mut self, reason: Option<String>) {
        self.paused = true;
        if reason.is_some() {
            self.last_error = reason;
        }
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.error_count = 0;
        self.last_error = None;
    }

    pub fn stop(&mut self) {
        self.queue.clear();
        self.active = None;
        self.error_count = 0;
        self.paused = false;
        self.last_error = None;
        self.last_failed_job = None;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn snapshot(&self) -> SyncStatusSnapshot {
        SyncStatusSnapshot {
            queue_len: self.queue.len(),
            active: self.active.clone(),
            queue_entries: self.queue.iter().cloned().collect(),
            last_pull: self.last_pull,
            last_push: self.last_push,
            paused: self.paused,
            error_count: self.error_count,
            last_error: self.last_error.clone(),
        }
    }

    pub fn clear_last_error(&mut self) {
        self.last_error = None;
    }

    pub fn take_last_failed_job(&mut self) -> Option<SyncJob> {
        self.last_failed_job.take()
    }

    fn is_duplicate(&self, kind: SyncJobKind) -> bool {
        if self.active.as_ref().is_some_and(|job| job.kind == kind) {
            return true;
        }
        self.queue.iter().any(|job| job.kind == kind)
    }
}

fn backoff_for(retries: u8) -> std::time::Duration {
    let secs = match retries {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => 4,
    };
    std::time::Duration::from_secs(secs)
}
