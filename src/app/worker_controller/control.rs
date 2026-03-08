use super::*;

impl WorkerController {
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
}
