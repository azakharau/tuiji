use ratatui::Frame;

use crate::app::{key_handlers::KeyHandler, state::ScreenType};

pub mod current_sprint;
pub mod home;

pub trait Screen: KeyHandler {
    fn draw(&mut self, frame: &mut Frame);

    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenState {
    Stay,
    SwitchTo(ScreenType),
    Quit,
    Refresh,
    OneRowUp,
    OneRowDown,
    OneColumnLeft,
    OneColumnRight,
    HalfPageUp,
    HalfPageDown,
    GoToTop,
    GoToBottom,
    GoToLine(usize),
}
