use crossterm::event::KeyCode;

use crate::ui::screens::ScreenState;

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
}

impl From<Mode> for &'static str {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
}

impl Mode {
    pub fn label(self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ScreenType {
    #[default]
    Home,
    BoardSelection,
    CurrentSprint,
    IssueDetail,
    MyIssues,
    SearchIssues,
    NewIssue,
    Conflicts,
    SyncStatus,
    Settings,
    SettingsThemes,
    SettingsThemeForm,
    ProfileCreation,
    Profiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionId {
    Quit,
    Refresh,
    Confirm,
    GoHome,
    OpenCurrentSprint,
    OpenMyIssues,
    OpenSearchIssues,
    OpenNewIssue,
    OpenProfiles,
    OpenBoards,
    OpenSettings,
    OpenSyncStatus,
    ResolveConflictLocal,
    ResolveConflictRemote,
    SyncNow,
    SyncPause,
    SyncRetry,
    SyncResume,
    FilterAll,
    FilterPull,
    FilterPush,
    NewProfile,
    EditProfile,
    DeleteProfile,
    OpenInBrowser,
    EditIssue,
    TransitionIssue,
    AddComment,
    AssignToMe,
    FocusQuery,
    PageUp,
    PageDown,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveTop,
    MoveBottom,
    MoveLineStart,
    MoveLineEnd,
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    EnterInsert(InsertMode),
    RawInput(KeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertMode {
    Before,
    After,
    LineStart,
    LineEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub action: ActionId,
    /// For motions, `repeat` tells how many times to perform the action.
    pub repeat: usize,
}

pub trait KeyHandler {
    fn handle_command(&mut self, command: Command) -> ScreenState;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionHint {
    pub binding: String,
    pub description: String,
}

impl ActionHint {
    pub fn render(&self) -> String {
        format!("[{}]{}", self.binding, self.description)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardRequiredBindings<'a> {
    pub open: &'a str,
    pub profiles: Option<&'a str>,
    pub quit: Option<&'a str>,
}
