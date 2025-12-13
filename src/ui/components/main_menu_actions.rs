use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};

use crate::{app::key_handlers::ActionHint, ui::utils::text_params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeMenuActions(Vec<ActionHint>);

impl Default for HomeMenuActions {
    fn default() -> Self {
        HomeMenuActions(vec![
            ActionHint {
                binding: "c".to_string(),
                description: "Current Sprint".to_string(),
            },
            ActionHint {
                binding: "i".to_string(),
                description: "My issues".to_string(),
            },
            ActionHint {
                binding: "s".to_string(),
                description: "Search Issues".to_string(),
            },
            ActionHint {
                binding: "n".to_string(),
                description: "New Issue".to_string(),
            },
            ActionHint {
                binding: "r".to_string(),
                description: "Refresh".to_string(),
            },
            ActionHint {
                binding: "p".to_string(),
                description: "Profiles".to_string(),
            },
            ActionHint {
                binding: "q".to_string(),
                description: "Quit".to_string(),
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
