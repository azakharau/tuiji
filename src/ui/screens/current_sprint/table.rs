use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use crate::ui::context::RenderContext;

use super::state::CurrentSprintState;

pub struct TableView;

impl TableView {
    pub fn draw(
        frame: &mut Frame,
        area: Rect,
        _state: &CurrentSprintState,
        context: &RenderContext,
    ) {
        let title = Paragraph::new(Line::from("Sprint Issues").centered())
            .style(Style::default().fg(context.colors().accent))
            .alignment(Alignment::Center);
        frame.render_widget(title, area);
    }
}
