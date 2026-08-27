use serde::{Deserialize, Serialize};

use super::keybinding_defaults;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct KeyBindingsConfig {
    #[serde(default)]
    pub global: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub home: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub board_selection: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub current_sprint: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issue_detail: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profile_creation: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub my_issues: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_issues: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub new_issue: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sync_status: Vec<KeyBindingConfig>,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        keybinding_defaults::default_keybindings_config()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct KeyBindingConfig {
    pub action: BindingAction,
    pub binding: String,
}

impl KeyBindingConfig {
    pub(super) fn new(action: BindingAction, binding: &str) -> Self {
        Self {
            action,
            binding: binding.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BindingAction {
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
    EnterInsertBefore,
    EnterInsertAfter,
    EnterInsertLineStart,
    EnterInsertLineEnd,
}
