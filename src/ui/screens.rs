use std::sync::Arc;

use ratatui::Frame;

use crate::{
    config::{CustomThemeConfig, ProfileConfig},
    ui::{
        context::RenderContext,
        interaction::{ActionHint, KeyHandler, Mode, ScreenType},
    },
};

pub mod board_selection;
pub mod conflicts;
pub mod current_sprint;
pub mod home;
pub mod issue_form;
pub mod issues_table;
pub mod my_issues;
pub mod profile_creation;
pub mod profiles;
pub mod search_issues;
pub mod settings;
pub mod sync_status;

pub trait Screen: KeyHandler {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext);

    fn name(&self) -> &'static str;

    /// Update action hints displayed in bottom bar (default no-op).
    fn set_action_hints(&mut self, _actions: Arc<Vec<ActionHint>>) {}

    /// Update current mode when screens display it (default no-op).
    fn set_mode(&mut self, _mode: Mode) {}

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

#[derive(Debug, Clone)]
pub enum ScreenState {
    Stay,
    Refresh,
    SwitchTo(ScreenType),
    SwitchMode(Mode),
    Quit,
    SaveProfile(ProfileConfig),
    SaveProfileAndClose(ProfileConfig),
    ApplyTheme(String),
    SaveCustomTheme(CustomThemeConfig),
    SaveCustomThemeAndClose(CustomThemeConfig),
    ResolveConflictLocal(String),
    ResolveConflictRemote(String),
    SyncNow,
    SyncPause,
    SyncRetry,
    SyncResume,
    Close,
}

impl PartialEq for ScreenState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Stay, Self::Stay) => true,
            (Self::Refresh, Self::Refresh) => true,
            (Self::SwitchTo(a), Self::SwitchTo(b)) => a == b,
            (Self::SwitchMode(a), Self::SwitchMode(b)) => a == b,
            (Self::Quit, Self::Quit) => true,
            (Self::SaveProfile(a), Self::SaveProfile(b)) => a == b,
            (Self::SaveProfileAndClose(a), Self::SaveProfileAndClose(b)) => a == b,
            (Self::ApplyTheme(a), Self::ApplyTheme(b)) => a == b,
            (Self::SaveCustomTheme(a), Self::SaveCustomTheme(b)) => a == b,
            (Self::SaveCustomThemeAndClose(a), Self::SaveCustomThemeAndClose(b)) => a == b,
            (Self::ResolveConflictLocal(a), Self::ResolveConflictLocal(b)) => a == b,
            (Self::ResolveConflictRemote(a), Self::ResolveConflictRemote(b)) => a == b,
            (Self::SyncNow, Self::SyncNow) => true,
            (Self::SyncPause, Self::SyncPause) => true,
            (Self::SyncRetry, Self::SyncRetry) => true,
            (Self::SyncResume, Self::SyncResume) => true,
            (Self::Close, Self::Close) => true,
            _ => false,
        }
    }
}
