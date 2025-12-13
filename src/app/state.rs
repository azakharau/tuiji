use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    Insert,
    Command,
}

impl From<Mode> for &'static str {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
}

impl Mode {
    pub fn as_paragraph(self) -> Paragraph<'static> {
        match self {
            Mode::Normal => {
                let val: &'static str = self.into();
                Paragraph::new(val)
                    .style(Style::default().fg(Color::Black))
                    .centered()
            }

            Mode::Visual => {
                let val: &'static str = self.into();
                Paragraph::new(val)
                    .style(Style::default().fg(Color::Black))
                    .centered()
            }
            Mode::Insert => {
                let val: &'static str = self.into();
                Paragraph::new(val)
                    .style(Style::default().fg(Color::Black))
                    .centered()
            }
            Mode::Command => {
                let val: &'static str = self.into();
                Paragraph::new(val)
                    .style(Style::default().fg(Color::Black))
                    .centered()
            }
        }
    }
}

impl Widget for Mode {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let border = Block::default().borders(Borders::NONE);
        let (content, color) = match self {
            Self::Normal => (self.as_paragraph(), Style::default().bg(Color::Blue)),
            Self::Insert => (self.as_paragraph(), Style::default().bg(Color::LightGreen)),
            Self::Visual => (
                self.as_paragraph(),
                Style::default().bg(Color::LightMagenta),
            ),
            Self::Command => (self.as_paragraph(), Style::default().bg(Color::Yellow)),
        };
        border.style(color).render(area, buf);
        content.render(area, buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScreenType {
    #[default]
    Home,
    CurrentSprint,
    MyIssues,
    SearchIssues,
    NewIssue,
    Profiles,
}
