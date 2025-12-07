use crate::ui::{
    components::{logo::AsciiLogoComponent, main_menu_actions::HomeMenuActions},
    screens::{Screen, ScreenAction},
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
    fn handle_key_event(&mut self, key_code: crossterm::event::KeyEvent) -> ScreenAction {
        match key_code.code {
            crossterm::event::KeyCode::Char('c') => {
                ScreenAction::SwitchTo(crate::app::state::ScreenType::CurrentSprint)
            }
            crossterm::event::KeyCode::Char('i') => {
                ScreenAction::SwitchTo(crate::app::state::ScreenType::MyIssues)
            }
            crossterm::event::KeyCode::Char('s') => {
                ScreenAction::SwitchTo(crate::app::state::ScreenType::SearchIssues)
            }
            crossterm::event::KeyCode::Char('n') => {
                ScreenAction::SwitchTo(crate::app::state::ScreenType::NewIssue)
            }
            crossterm::event::KeyCode::Char('r') => ScreenAction::Refresh,
            crossterm::event::KeyCode::Char('p') => {
                ScreenAction::SwitchTo(crate::app::state::ScreenType::Profiles)
            }
            crossterm::event::KeyCode::Char('q') => ScreenAction::Quit,
            _ => ScreenAction::Stay,
        }
    }
    fn name(&self) -> &'static str {
        "HomeScreen"
    }
}
