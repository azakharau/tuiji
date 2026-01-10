use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
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
    pub fn color(self) -> Color {
        match self {
            Self::Normal => Color::Blue,
            Self::Insert => Color::LightGreen,
            Self::Visual => Color::LightMagenta,
            Self::Command => Color::Yellow,
        }
    }

    pub fn style(self) -> Style {
        Style::default().bg(self.color()).fg(Color::Black)
    }

    pub fn as_paragraph(self) -> Paragraph<'static> {
        let val: &'static str = self.into();
        Paragraph::new(val)
            .style(Style::default().fg(Color::Black))
            .centered()
    }
}

impl Widget for Mode {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let border = Block::default().borders(Borders::NONE);
        border.style(self.style()).render(area, buf);
        let content = self.as_paragraph();
        content.render(area, buf);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ScreenType {
    #[default]
    Home,
    BoardSelection,
    CurrentSprint,
    MyIssues,
    SearchIssues,
    NewIssue,
    Settings,
    SettingsThemes,
    SettingsThemeForm,
    ProfileCreation,
    Profiles,
}
