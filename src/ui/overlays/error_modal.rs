use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Style,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    app::error::{AppErrorLevel, AppErrorState},
    app::input::overlay::modal_area,
    ui::{components::layout::ModalFrame, context::RenderContext},
};

pub struct ErrorModal<'a> {
    error: &'a AppErrorState,
    context: &'a RenderContext,
}

impl<'a> ErrorModal<'a> {
    pub fn new(error: &'a AppErrorState, context: &'a RenderContext) -> Self {
        Self { error, context }
    }
}

impl Widget for ErrorModal<'_> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let height = 6.min(area.height);
        let modal = modal_area(area, 60.min(area.width), height);
        let colors = self.context.colors();
        let color = match self.error.level {
            AppErrorLevel::Error => colors.error,
            AppErrorLevel::Warning => colors.warning,
            AppErrorLevel::Info => colors.info,
        };
        let inner = ModalFrame::new(
            self.error.title.as_str(),
            modal,
            Style::default().fg(color).bg(self.context.colors().background),
            self.context,
        )
        .render_to_buffer(buf);
        let sections = Layout::vertical([Constraint::Fill(1)]).split(inner);
        let text = Paragraph::new(self.error.message.as_str())
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        text.render(sections[0], buf);
    }
}
