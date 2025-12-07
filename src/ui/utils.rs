use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

pub struct Width(u16);

impl Width {
    pub fn new(item: u16) -> Self {
        Width(item)
    }
    pub fn get(&self) -> u16 {
        self.0
    }
}

pub struct Height(u16);

impl Height {
    pub fn new(item: u16) -> Self {
        Height(item)
    }
    pub fn get(&self) -> u16 {
        self.0
    }
}

pub fn text_params(text: &str) -> (Width, Height) {
    let lines: Vec<&str> = text.lines().collect();
    let text_height = lines.len() as u16;
    let text_width = lines
        .into_iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;
    (Width(text_width), Height(text_height))
}

pub fn render_centered(
    content: impl Widget,
    area: Rect,
    width: Width,
    height: Height,
    buf: &mut Buffer,
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((area.height.saturating_sub(height.0)) / 2),
            Constraint::Length(height.0),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((area.width.saturating_sub(width.0)) / 2),
            Constraint::Length(width.0),
            Constraint::Min(0),
        ])
        .split(vertical[1]);
    content.render(horizontal[1], buf);
}
