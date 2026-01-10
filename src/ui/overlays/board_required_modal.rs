use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    app::{input::overlay::modal_area, overlay::BoardRequiredBindings},
    ui::{components::layout::ModalFrame, context::RenderContext},
};

pub struct BoardRequiredModal<'a> {
    bindings: &'a BoardRequiredBindings<'a>,
    context: &'a RenderContext,
}

impl<'a> BoardRequiredModal<'a> {
    pub fn new(bindings: &'a BoardRequiredBindings<'a>, context: &'a RenderContext) -> Self {
        Self { bindings, context }
    }
}

impl Widget for BoardRequiredModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = 7.min(area.height);
        let modal = modal_area(area, 60.min(area.width), height);
        let border_style = Style::default()
            .fg(self.context.colors().warning)
            .bg(self.context.colors().background);
        let inner =
            ModalFrame::new("Board Required", modal, border_style, self.context)
                .render_to_buffer(buf);
        let sections = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(inner);
        let text = Paragraph::new("No board selected.\nConfigure a board to continue.")
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        text.render(sections[0], buf);
        let mut options = vec![format!("[{}] Configure boards", self.bindings.open)];
        if let Some(profile_key) = self.bindings.profiles {
            options.push(format!("[{profile_key}] Profiles"));
        }
        options.push(format!("[{}] Quit", self.bindings.quit));
        let options = Paragraph::new(options.join("\n"))
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        options.render(sections[1], buf);
    }
}
