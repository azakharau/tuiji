use crossterm::event::KeyEvent;

use crate::{
    app::{state::ScreenType, worker_controller::SyncJob},
    config::ProfileConfig,
};

/// Unified event type consumed by the main loop.
pub enum AppEvent {
    Input(InputEvent),
    Ui(UiEvent),
    Nav(NavEvent),
    Repo(RepoEvent),
    Worker(WorkerEvent),
    Notification(NotificationEvent),
    System(SystemEvent),
}

pub enum InputEvent {
    Key(KeyEvent),
}

pub enum UiEvent {
    Render,
    Error(String),
}

pub enum NavEvent {
    SwitchTo(ScreenType),
    Back,
    Quit,
}

pub enum RepoEvent {
    SaveProfile(ProfileConfig),
    DeleteProfile(String),
    SelectBoard(u64),
}

/// Messages produced by background workers (placeholder for Jira/cache notifications).
pub enum WorkerEvent {
    JiraUpdated,
    Notification(String),
    SyncCompleted(SyncJob),
    SyncFailed { job: SyncJob, error: String },
}

pub enum NotificationEvent {
    Message(String),
}

pub enum SystemEvent {
    Tick,
}
