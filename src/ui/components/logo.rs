use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};

const LOGO: &str = r"
░██████████░██     ░██ ░██████    ░█████ ░██████
    ░██    ░██     ░██   ░██        ░██    ░██
    ░██    ░██     ░██   ░██        ░██    ░██
    ░██    ░██     ░██   ░██        ░██    ░██
    ░██    ░██     ░██   ░██  ░██   ░██    ░██
    ░██     ░██   ░██    ░██  ░██   ░██    ░██
    ░██      ░██████   ░██████ ░██████   ░██████
";

#[derive(Debug, Clone)]
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
        let lines: Vec<&str> = self.0.lines().collect();
        let logo_height = lines.len() as u16;
        let logo_width = lines
            .into_iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as u16;
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min((area.height.saturating_sub(logo_height)) / 2),
                Constraint::Length(logo_height),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min((area.width.saturating_sub(logo_width)) / 2),
                Constraint::Length(logo_width),
                Constraint::Min(0),
            ])
            .split(vertical[1]);
        Text::raw(self.0).render(horizontal[1], buf);
        Text::raw("Like JIRA but in your terminal")
            .centered()
            .render(vertical[3], buf);
    }
}
