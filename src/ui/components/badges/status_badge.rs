use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::ui::context::RenderContext;

pub enum StatusVariant<'a> {
    Todo,
    InProgress,
    Done,
    Custom(&'a str),
}

pub struct StatusBadge<'a> {
    label: &'a str,
    variant: StatusVariant<'a>,
    context: &'a RenderContext,
}

impl<'a> StatusBadge<'a> {
    pub fn new(label: &'a str, variant: StatusVariant<'a>, context: &'a RenderContext) -> Self {
        Self {
            label,
            variant,
            context,
        }
    }

    fn color(&self) -> Color {
        let colors = self.context.colors();
        match self.variant {
            StatusVariant::Todo => colors.info,
            StatusVariant::InProgress => colors.warning,
            StatusVariant::Done => colors.success,
            StatusVariant::Custom(_) => colors.logo,
        }
    }

    fn text(&self) -> &str {
        match self.variant {
            StatusVariant::Custom(label) => label,
            _ => self.label,
        }
    }
}

impl Widget for StatusBadge<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(self.color());
        let line = Line::from(self.text()).centered();
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .style(style)
            .render(area, buf);
    }
}
