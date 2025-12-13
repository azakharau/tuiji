use std::sync::Arc;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

use crate::app::{key_handlers::ActionHint, state::Mode};

pub struct BottomBar {
    pub mode: Mode,
    pub actions: Arc<Vec<ActionHint>>,
}

impl BottomBar {
    pub fn new(mode: Mode, actions: Arc<Vec<ActionHint>>) -> Self {
        BottomBar { mode, actions }
    }
}

impl Widget for BottomBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let actions_str = self
            .actions
            .iter()
            .map(|action| action.render())
            .collect::<Vec<String>>()
            .join(" ");
        let actions_paragraph = Paragraph::new(actions_str)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        let chunks = Layout::horizontal([Constraint::Length(10), Constraint::Min(0)]).split(area);

        self.mode.render(chunks[0], buf);
        actions_paragraph.render(chunks[1], buf);
    }
}
