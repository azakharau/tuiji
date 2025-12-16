use crate::{
    app::key_handlers::{ActionId, Command, KeyHandler},
    ui::{
        components::{logo::AsciiLogoComponent, main_menu_actions::HomeMenuActions},
        screens::{Screen, ScreenState},
    },
};

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    widgets::Widget,
};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct HomeScreen {
    logo: AsciiLogoComponent,
    actions: HomeMenuActions,
}

impl Widget for HomeScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.logo.render(area, buf);
    }
}

impl Screen for HomeScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let screen_layout = Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        // TODO: Consider rework to use borrowing instead of cloning
        frame.render_widget(self.logo.clone(), screen_layout[1]);
        frame.render_widget(self.actions.clone(), screen_layout[2]);
    }
    fn name(&self) -> &'static str {
        "Home Screen"
    }
}

impl KeyHandler for HomeScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::Refresh => ScreenState::Refresh,
            _ => ScreenState::Stay,
        }
    }
}
