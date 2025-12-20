use std::sync::Arc;

use ratatui::Frame;

use crate::app::{
    key_handlers::{ActionHint, KeyHandler},
    state::ScreenType,
};

pub mod current_sprint;
pub mod home;
pub mod profile_creation;

pub trait Screen: KeyHandler {
    fn draw(&mut self, frame: &mut Frame);

    fn name(&self) -> &'static str;

    /// Update action hints displayed in bottom bar (default no-op).
    fn set_action_hints(&mut self, _actions: Arc<Vec<ActionHint>>) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenState {
    Stay,
    Refresh,
    SwitchTo(ScreenType),
    Quit,
}
