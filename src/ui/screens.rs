use std::sync::Arc;

use ratatui::Frame;

use crate::{
    app::{
        key_handlers::{ActionHint, KeyHandler},
        state::ScreenType,
    },
    config::ProfileConfig,
};

pub mod board_selection;
pub mod current_sprint;
pub mod home;
pub mod profile_creation;
pub mod profiles;

pub trait Screen: KeyHandler {
    fn draw(&mut self, frame: &mut Frame);

    fn name(&self) -> &'static str;

    /// Update action hints displayed in bottom bar (default no-op).
    fn set_action_hints(&mut self, _actions: Arc<Vec<ActionHint>>) {}

    /// Update current mode when screens display it (default no-op).
    fn set_mode(&mut self, _mode: crate::app::state::Mode) {}

    /// Handle command-line actions like :w or :wq.
    fn handle_command_line(&mut self, _cmd: CommandLineCommand) -> ScreenState {
        ScreenState::Stay
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandLineCommand {
    Write,
    WriteQuit,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenState {
    Stay,
    Refresh,
    SwitchTo(ScreenType),
    Quit,
    SaveProfile(ProfileConfig),
    SaveProfileAndClose(ProfileConfig),
    Close,
}
