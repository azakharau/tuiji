use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};

use crate::{app::key_handlers::ActionItem, ui::utils::text_params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeMenuActions(Vec<ActionItem>);

impl Default for HomeMenuActions {
    fn default() -> Self {
        HomeMenuActions(vec![
            ActionItem {
                binding: "c",
                description: "Current Sprint",
            },
            ActionItem {
                binding: "i",
                description: "My issues",
            },
            ActionItem {
                binding: "s",
                description: "Search Issues",
            },
            ActionItem {
                binding: "n",
                description: "New Issue",
            },
            ActionItem {
                binding: "r",
                description: "Refresh",
            },
            ActionItem {
                binding: "p",
                description: "Profiles",
            },
            ActionItem {
                binding: "q",
                description: "Quit",
            },
        ])
    }
}

impl Widget for HomeMenuActions {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let actions_str = self
            .0
            .into_iter()
            .map(|action| action.render())
            .collect::<Vec<String>>()
            .join("\n");
        let (width, _) = text_params(&actions_str);
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min((area.width.saturating_sub(width.get())) / 2),
                Constraint::Length(width.get()),
                Constraint::Min(0),
            ])
            .split(area);
        let text = Text::raw(actions_str);
        text.render(horizontal[1], buf);
    }
}
