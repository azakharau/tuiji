use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::ui::context::RenderContext;

pub struct EmptyState<'a> {
    title: &'a str,
    message: &'a str,
    context: &'a RenderContext,
}

impl<'a> EmptyState<'a> {
    pub fn new(title: &'a str, message: &'a str, context: &'a RenderContext) -> Self {
        Self {
            title,
            message,
            context,
        }
    }
}

impl Widget for EmptyState<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let style = Style::default()
            .fg(self.context.colors().info)
            .bg(self.context.colors().background);
        let text = Line::from(format!("{}\n{}", self.title, self.message)).centered();
        Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center)
            .wrap(Wrap::default())
            .render(area, buf);
    }
}
