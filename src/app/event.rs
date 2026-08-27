use crossterm::event::KeyEvent;

use crate::contracts::sync::SyncJob;

/// Unified event type consumed by the main loop.
pub enum AppEvent {
    Input(InputEvent),
    Ui(UiEvent),
    Worker(WorkerEvent),
    System(SystemEvent),
}

pub enum InputEvent {
    Key(KeyEvent),
}

pub enum UiEvent {
    Error(String),
}

/// Messages produced by background workers (placeholder for Jira/cache notifications).
pub enum WorkerEvent {
    Notification(String),
    SyncCompleted(SyncJob),
    SyncFailed { job: SyncJob, error: String },
}

pub enum SystemEvent {
    Tick,
}
