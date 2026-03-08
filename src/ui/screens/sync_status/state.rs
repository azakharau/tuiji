use crate::{
    contracts::sync::SyncStatusSnapshot,
    data::{SyncLogEntry, SyncLogFilter},
};

pub struct SyncStatusState {
    snapshot: SyncStatusSnapshot,
    sync_log: Vec<SyncLogEntry>,
    filter: SyncLogFilter,
}

impl SyncStatusState {
    pub fn new(
        snapshot: SyncStatusSnapshot,
        sync_log: Vec<SyncLogEntry>,
        filter: SyncLogFilter,
    ) -> Self {
        Self {
            snapshot,
            sync_log,
            filter,
        }
    }

    pub fn snapshot(&self) -> &SyncStatusSnapshot {
        &self.snapshot
    }

    pub fn set_snapshot(&mut self, snapshot: SyncStatusSnapshot) {
        self.snapshot = snapshot;
    }

    pub fn sync_log(&self) -> &[SyncLogEntry] {
        &self.sync_log
    }

    pub fn set_sync_log(&mut self, entries: Vec<SyncLogEntry>) {
        self.sync_log = entries;
    }

    pub fn filter(&self) -> SyncLogFilter {
        self.filter
    }

    pub fn set_filter(&mut self, filter: SyncLogFilter) {
        self.filter = filter;
    }
}
