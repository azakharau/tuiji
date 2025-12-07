use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::app::state::ScreenType;

pub mod home;

pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);

    fn handle_key_event(&mut self, key_code: KeyEvent) -> ScreenAction;

    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenAction {
    Stay,
    SwitchTo(ScreenType),
    Quit,
    Refresh,
}
