use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::{
    app::key_handlers::{ActionId, Command, KeyHandler},
    ui::screens::{Screen, ScreenState},
};

pub struct ProfileScreen;

impl ProfileScreen {
    pub fn new(_cfg: crate::config::AppConfig) -> Self { ProfileScreen }
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
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::Refresh => ScreenState::Refresh,
            ActionId::Quit => ScreenState::Quit,
            _ => ScreenState::Stay,
        }
    }
}
