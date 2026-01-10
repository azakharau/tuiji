use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
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
pub struct AsciiLogoComponent {
    text: &'static str,
    style: Style,
}

impl Default for AsciiLogoComponent {
    fn default() -> Self {
        AsciiLogoComponent {
            text: LOGO,
            style: Style::default(),
        }
    }
}

impl AsciiLogoComponent {
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for AsciiLogoComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (logo_width, logo_height) = text_params(self.text);
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
        Text::styled(self.text, self.style).render(horizontal[1], buf);
        Text::styled("Like JIRA but in your terminal", self.style)
            .centered()
            .render(vertical[3], buf);
    }
}
