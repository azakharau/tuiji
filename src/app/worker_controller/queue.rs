use super::*;

impl WorkerController {
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

    fn is_duplicate(&self, kind: SyncJobKind) -> bool {
        if self.active.as_ref().is_some_and(|job| job.kind == kind) {
            return true;
        }
        self.queue.iter().any(|job| job.kind == kind)
    }
}
