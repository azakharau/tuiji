use std::time::{Duration, Instant};

use crate::contracts::error::AppErrorLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppNotificationKind {
    System,
    Reminder,
    Info,
}

impl AppNotificationKind {
    pub fn priority(self) -> u8 {
        match self {
            AppNotificationKind::System => 3,
            AppNotificationKind::Reminder => 2,
            AppNotificationKind::Info => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AppNotificationKind::System => "System",
            AppNotificationKind::Reminder => "Reminder",
            AppNotificationKind::Info => "Info",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppNotification {
    pub message: String,
    pub level: AppErrorLevel,
    pub kind: AppNotificationKind,
    pub expires_at: Instant,
}

impl AppNotification {
    pub fn new(
        message: impl Into<String>,
        level: AppErrorLevel,
        kind: AppNotificationKind,
        ttl: Duration,
    ) -> Self {
        Self {
            message: message.into(),
            level,
            kind,
            expires_at: Instant::now() + ttl,
        }
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires_at
    }
}
