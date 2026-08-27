use super::*;

impl WorkerController {
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

    pub fn last_pull(&self) -> Option<SystemTime> {
        self.last_pull
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
