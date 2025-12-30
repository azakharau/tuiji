use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct CommandLine<'a> {
    pub buffer: &'a str,
}

impl<'a> CommandLine<'a> {
    pub fn new(buffer: &'a str) -> Self {
        Self { buffer }
    }
}

impl Widget for CommandLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let paragraph =
            Paragraph::new(self.buffer).style(Style::default().fg(Color::Yellow));
        paragraph.render(area, buf);

        if area.width == 0 || area.height == 0 {
            return;
        }
        let text_width = self.buffer.chars().count() as u16;
        let cursor_x = area.x + text_width.min(area.width.saturating_sub(1));
        let cursor = Rect {
            x: cursor_x,
            y: area.y,
            width: 1,
            height: 1,
        };
        let cursor_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::Yellow).fg(Color::Black));
        cursor_block.render(cursor, buf);
    }
}
