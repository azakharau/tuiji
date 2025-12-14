use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::{
    app::key_handlers::KeyHandler,
    config::AppConfig,
    ui::screens::{Screen, ScreenState},
};

pub struct ProfileScreen {
    conf: AppConfig,
}

impl ProfileScreen {
    pub fn new(conf: AppConfig) -> Self {
        ProfileScreen { conf }
    }
}

impl Screen for ProfileScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let paragraph = Paragraph::new("Profile Picker Screen - Not Implemented Yet")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, frame.area());
    }

    fn name(&self) -> &'static str {
        "Profile Picker"
    }
}

impl KeyHandler for ProfileScreen {
    fn handle_command(&mut self, command: crate::app::key_handlers::Command) -> ScreenState {
        use crate::app::key_handlers::Command;
        match command {
            Command::Refresh => ScreenState::Refresh,
            Command::SwitchTo(screen) => ScreenState::SwitchTo(screen),
            Command::Unhandled(key) => match key.code {
                crossterm::event::KeyCode::Char('q') => ScreenState::Quit,
                _ => ScreenState::Stay,
            },
            Command::Quit => ScreenState::Quit,
            _ => ScreenState::Stay,
        }
    }
}
