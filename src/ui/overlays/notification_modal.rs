use std::collections::VecDeque;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
};

use crate::{
    app::input::overlay::notification_area,
    app::notification::{AppNotification, AppNotificationKind},
    ui::context::RenderContext,
};

pub struct NotificationModal<'a> {
    notifications: &'a VecDeque<AppNotification>,
    context: &'a RenderContext,
}

impl<'a> NotificationModal<'a> {
    pub fn new(notifications: &'a VecDeque<AppNotification>, context: &'a RenderContext) -> Self {
        Self {
            notifications,
            context,
        }
    }

    fn kind_color(&self, kind: AppNotificationKind) -> Color {
        let colors = self.context.colors();
        match kind {
            AppNotificationKind::System => colors.error,
            AppNotificationKind::Reminder => colors.warning,
            AppNotificationKind::Info => colors.info,
        }
    }

    fn title_for(kind: AppNotificationKind, total: usize) -> &'static str {
        if total == 1 {
            match kind {
                AppNotificationKind::System => "System Notice",
                AppNotificationKind::Reminder => "Reminder",
                AppNotificationKind::Info => "Notification",
            }
        } else {
            match kind {
                AppNotificationKind::System => "System Notice",
                AppNotificationKind::Reminder => "Reminder",
                AppNotificationKind::Info => "Notifications",
            }
        }
    }

    fn dominant_kind(system: usize, reminder: usize, info: usize) -> AppNotificationKind {
        let mut best = AppNotificationKind::Info;
        let mut best_count = info;
        let mut consider = |kind: AppNotificationKind, count: usize| {
            if count > best_count || (count == best_count && kind.priority() > best.priority()) {
                best = kind;
                best_count = count;
            }
        };
        consider(AppNotificationKind::Reminder, reminder);
        consider(AppNotificationKind::System, system);
        best
    }
}

impl Widget for NotificationModal<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut items: Vec<ListItem<'_>> = Vec::new();
        let mut total = 0usize;
        let mut system = 0usize;
        let mut reminder = 0usize;
        let mut info = 0usize;

        for note in self.notifications.iter() {
            total += 1;
            match note.kind {
                AppNotificationKind::System => system += 1,
                AppNotificationKind::Reminder => reminder += 1,
                AppNotificationKind::Info => info += 1,
            }
            let color = self.kind_color(note.kind);
        let style = Style::default()
            .fg(color)
            .bg(self.context.colors().background);
            let line = Line::from(vec![
                Span::raw("["),
                Span::styled(note.kind.label(), style),
                Span::raw("] "),
                Span::styled(note.message.as_str(), style),
            ])
            .alignment(Alignment::Center);
            if !items.is_empty() {
                items.push(ListItem::new(Line::raw("")));
            }
            items.push(ListItem::new(line));
        }

        let dominant = if total == 0 {
            AppNotificationKind::Info
        } else {
            Self::dominant_kind(system, reminder, info)
        };
        let height = (total as u16 * 2 + 5).min(area.height).max(6);
        let width = (area.width.saturating_mul(3) / 10)
            .max(24)
            .min(area.width);
        let modal = notification_area(area, width, height);
        let border_color = self.kind_color(dominant);
        let title = if total == 0 {
            "Notifications"
        } else {
            Self::title_for(dominant, total)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(
                Style::default()
                    .fg(self.context.colors().text)
                    .bg(self.context.colors().background),
            )
            .title(Line::from(title).centered())
            .title_style(Style::default().fg(border_color));
        Clear.render(modal, buf);
        let inner = block.inner(modal);
        block.render(modal, buf);

        let list = List::new(items);
        list.render(inner, buf);
    }
}
