use std::{collections::VecDeque, time::Duration};

use crate::{
    app::{
        error::{AppErrorLevel, AppErrorState},
        notification::{AppNotification, AppNotificationKind},
    },
    config::UiConfig,
};

pub struct NotificationService {
    error_state: Option<AppErrorState>,
    error_expires_at: Option<std::time::Instant>,
    notifications: VecDeque<AppNotification>,
    notification_ttl: Duration,
    notification_stack_limit: usize,
    error_ttl: Duration,
}

impl NotificationService {
    pub fn from_ui(ui: &UiConfig) -> Self {
        Self::new(
            Duration::from_secs(ui.notification_ttl_seconds),
            ui.notification_stack_limit,
            Duration::from_secs(ui.error_ttl_seconds),
        )
    }

    pub fn new(
        notification_ttl: Duration,
        notification_stack_limit: usize,
        error_ttl: Duration,
    ) -> Self {
        Self {
            error_state: None,
            error_expires_at: None,
            notifications: VecDeque::new(),
            notification_ttl,
            notification_stack_limit,
            error_ttl,
        }
    }

    pub fn error_state(&self) -> Option<&AppErrorState> {
        self.error_state.as_ref()
    }

    pub fn has_error(&self) -> bool {
        self.error_state.is_some()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, AppNotification> {
        self.notifications.iter()
    }

    pub fn items(&self) -> &VecDeque<AppNotification> {
        &self.notifications
    }

    pub fn clear_error(&mut self) {
        self.error_state = None;
        self.error_expires_at = None;
    }

    pub fn set_error(&mut self, error: AppErrorState) {
        self.error_state = Some(error);
        if self.error_ttl.is_zero() {
            self.error_expires_at = None;
        } else {
            self.error_expires_at = Some(std::time::Instant::now() + self.error_ttl);
        }
    }

    pub fn push_notification(
        &mut self,
        message: String,
        level: AppErrorLevel,
        kind: AppNotificationKind,
    ) {
        if self.notification_stack_limit == 0 || self.notification_ttl.is_zero() {
            return;
        }
        if self.notifications.len() >= self.notification_stack_limit {
            self.notifications.pop_front();
        }
        self.notifications.push_back(AppNotification::new(
            message,
            level,
            kind,
            self.notification_ttl,
        ));
    }

    pub fn tick(&mut self) -> bool {
        let mut changed = false;
        if let Some(expires_at) = self.error_expires_at {
            if std::time::Instant::now() >= expires_at {
                self.error_state = None;
                self.error_expires_at = None;
                changed = true;
            }
        }
        if !self.notifications.is_empty() {
            let now = std::time::Instant::now();
            let before = self.notifications.len();
            self.notifications.retain(|note| !note.is_expired(now));
            if before != self.notifications.len() {
                changed = true;
            }
        }
        changed
    }
}
