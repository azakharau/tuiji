use crossterm::event::KeyEvent;

/// Unified event type consumed by the main loop.
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Worker(WorkerMessage),
}

/// Messages produced by background workers (placeholder for Jira/cache notifications).
pub enum WorkerMessage {
    JiraUpdated,
    Notification(String),
}
