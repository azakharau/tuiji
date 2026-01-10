use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Widget},
    Frame,
};

use crate::ui::context::RenderContext;

pub struct ModalFrame<'a> {
    title: &'a str,
    area: Rect,
    style: Style,
    context: &'a RenderContext,
}

impl<'a> ModalFrame<'a> {
    pub fn new(title: &'a str, area: Rect, style: Style, context: &'a RenderContext) -> Self {
        Self {
            title,
            area,
            style,
            context,
        }
    }

    pub fn render(self, frame: &mut Frame) -> Rect {
        let block = self.block();
        let inner = block.inner(self.area);
        frame.render_widget(Clear, self.area);
        frame.render_widget(block, self.area);
        inner
    }

    pub fn render_to_buffer(self, buf: &mut Buffer) -> Rect {
        let block = self.block();
        let inner = block.inner(self.area);
        Clear.render(self.area, buf);
        block.render(self.area, buf);
        inner
    }

    fn block(&self) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(self.style)
            .style(
                Style::default()
                    .fg(self.context.colors().text)
                    .bg(self.context.colors().background),
            )
            .title(Line::from(self.title).centered())
            .title_style(self.style)
    }
}
