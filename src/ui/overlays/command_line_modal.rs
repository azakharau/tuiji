use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::app::input::overlay::command_line_area;
use crate::ui::context::RenderContext;

pub struct CommandLineModal<'a> {
    buffer: &'a str,
    context: &'a RenderContext,
}

impl<'a> CommandLineModal<'a> {
    pub fn new(buffer: &'a str, context: &'a RenderContext) -> Self {
        Self { buffer, context }
    }
}

impl Widget for CommandLineModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let accent = self.context.colors().mode_command_bg;
        let area = command_line_area(area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent))
            .style(
                Style::default()
                    .fg(self.context.colors().text)
                    .bg(self.context.colors().background),
            )
            .title(Line::from("Command").centered())
            .title_style(Style::default().fg(accent));
        Clear.render(area, buf);
        let inner = block.inner(area);
        block.render(area, buf);
        CommandLineInput::new(self.buffer, accent, self.context.colors().background)
            .render(inner, buf);
    }
}

struct CommandLineInput<'a> {
    buffer: &'a str,
    color: Color,
    background: Color,
}

impl<'a> CommandLineInput<'a> {
    fn new(buffer: &'a str, color: Color, background: Color) -> Self {
        Self {
            buffer,
            color,
            background,
        }
    }
}

impl Widget for CommandLineInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(self.buffer)
            .style(Style::default().fg(self.color).bg(self.background));
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
            .style(Style::default().bg(self.color).fg(self.background));
        cursor_block.render(cursor, buf);
    }
}
