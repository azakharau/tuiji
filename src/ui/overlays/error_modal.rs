use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::{
    app::error::{AppErrorLevel, AppErrorState},
    app::input::overlay::modal_area,
};

pub struct ErrorModal<'a> {
    error: &'a AppErrorState,
}

impl<'a> ErrorModal<'a> {
    pub fn new(error: &'a AppErrorState) -> Self {
        Self { error }
    }
}

impl Widget for ErrorModal<'_> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let height = 6.min(area.height);
        let modal = modal_area(area, 60.min(area.width), height);
        let color = match self.error.level {
            AppErrorLevel::Error => Color::Red,
            AppErrorLevel::Warning => Color::Yellow,
            AppErrorLevel::Info => Color::Cyan,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(Line::from(self.error.title.as_str()).centered())
            .title_style(Style::default().fg(color));
        let inner = block.inner(modal);
        Clear.render(modal, buf);
        block.render(modal, buf);
        let sections = Layout::vertical([Constraint::Fill(1)]).split(inner);
        let text = Paragraph::new(self.error.message.as_str())
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        text.render(sections[0], buf);
    }
}
