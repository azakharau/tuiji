use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};

use crate::ui::utils::text_params;

const LOGO: &str = r"
░██████████░██     ░██ ░██████    ░█████ ░██████
    ░██    ░██     ░██   ░██        ░██    ░██
    ░██    ░██     ░██   ░██        ░██    ░██
    ░██    ░██     ░██   ░██        ░██    ░██
    ░██    ░██     ░██   ░██  ░██   ░██    ░██
    ░██     ░██   ░██    ░██  ░██   ░██    ░██
    ░██      ░██████   ░██████ ░██████   ░██████
";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AsciiLogoComponent(&'static str);

impl Default for AsciiLogoComponent {
    fn default() -> Self {
        AsciiLogoComponent(LOGO)
    }
}

impl Widget for AsciiLogoComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (logo_width, logo_height) = text_params(self.0);
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min((area.height.saturating_sub(logo_height.get())) / 2),
                Constraint::Length(logo_height.get()),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min((area.width.saturating_sub(logo_width.get())) / 2),
                Constraint::Length(logo_width.get()),
                Constraint::Min(0),
            ])
            .split(vertical[1]);
        Text::raw(self.0).render(horizontal[1], buf);
        Text::raw("Like JIRA but in your terminal")
            .centered()
            .render(vertical[3], buf);
    }
}
