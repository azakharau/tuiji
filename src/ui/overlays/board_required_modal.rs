use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::app::{input::overlay::modal_area, overlay::BoardRequiredBindings};

pub struct BoardRequiredModal<'a> {
    bindings: &'a BoardRequiredBindings<'a>,
    color: Color,
}

impl<'a> BoardRequiredModal<'a> {
    pub fn new(bindings: &'a BoardRequiredBindings<'a>, color: Color) -> Self {
        Self { bindings, color }
    }
}

impl Widget for BoardRequiredModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = 7.min(area.height);
        let modal = modal_area(area, 60.min(area.width), height);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color))
            .title(Line::from("Board Required").centered())
            .title_style(Style::default().fg(self.color));
        Clear.render(modal, buf);
        let inner = block.inner(modal);
        block.render(modal, buf);
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
