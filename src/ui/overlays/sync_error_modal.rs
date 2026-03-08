use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    ui::layout::modal_area,
    ui::{components::layout::ModalFrame, context::RenderContext},
};

pub struct SyncErrorModal<'a> {
    error: Option<&'a str>,
    context: &'a RenderContext,
}

impl<'a> SyncErrorModal<'a> {
    pub fn new(error: Option<&'a str>, context: &'a RenderContext) -> Self {
        Self { error, context }
    }
}

impl Widget for SyncErrorModal<'_> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let height = 8.min(area.height).max(5);
        let modal = modal_area(area, 72.min(area.width), height);
        let color = self.context.colors().error;
        let inner = ModalFrame::new(
            "Sync error",
            modal,
            Style::default()
                .fg(color)
                .bg(self.context.colors().background),
            self.context,
        )
        .render_to_buffer(buf);

        let sections = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(inner);
        let message = self.error.unwrap_or("Sync paused after repeated errors.");
        let text = Paragraph::new(message)
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        text.render(sections[0], buf);

        let hints = Line::from(vec![
            Span::styled("r", Style::default().fg(self.context.colors().success)),
            Span::raw(" = retry  "),
            Span::styled("q", Style::default().fg(self.context.colors().warning)),
            Span::raw(" = stop"),
        ])
        .alignment(Alignment::Center);
        Paragraph::new(hints).render(sections[1], buf);
    }
}
