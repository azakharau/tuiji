use crate::ui::interaction::{ActionId, ScreenType};

type ScreenHint = (ActionId, &'static str);

const HOME_HINTS: &[ScreenHint] = &[
    (ActionId::Quit, "Quit"),
    (ActionId::Confirm, "Select"),
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::OpenCurrentSprint, "Current sprint"),
    (ActionId::OpenMyIssues, "My issues"),
    (ActionId::OpenSearchIssues, "Search issues"),
    (ActionId::OpenNewIssue, "New issue"),
    (ActionId::OpenBoards, "Boards"),
    (ActionId::OpenSyncStatus, "Sync status"),
    (ActionId::OpenSettings, "Settings"),
];

const CURRENT_SPRINT_HINTS: &[ScreenHint] = &[
    (ActionId::Confirm, "Details"),
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::MoveTop, "Top"),
    (ActionId::MoveBottom, "Bottom"),
    (ActionId::GoHome, "Home"),
];

const ISSUE_DETAIL_HINTS: &[ScreenHint] = &[
    (ActionId::Quit, "Quit"),
    (ActionId::EditIssue, "Edit issue"),
    (ActionId::TransitionIssue, "Transition"),
    (ActionId::AddComment, "Comment"),
    (ActionId::AssignToMe, "Assign to me"),
    (ActionId::OpenInBrowser, "Open in browser"),
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::MoveTop, "Top"),
    (ActionId::MoveBottom, "Bottom"),
    (ActionId::PageUp, "Page up"),
    (ActionId::PageDown, "Page down"),
];

const ISSUES_WORKSPACE_HINTS: &[ScreenHint] = &[
    (ActionId::Confirm, "Details"),
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::MoveTop, "Top"),
    (ActionId::MoveBottom, "Bottom"),
    (ActionId::GoHome, "Home"),
];

const BOARD_SELECTION_HINTS: &[ScreenHint] =
    &[(ActionId::Quit, "Quit"), (ActionId::GoHome, "Home")];

const PROFILES_HINTS: &[ScreenHint] = &[
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::Confirm, "Activate"),
    (ActionId::EditProfile, "Edit"),
    (ActionId::DeleteProfile, "Delete"),
    (ActionId::NewProfile, "New"),
    (ActionId::GoHome, "Home"),
];

const SETTINGS_HINTS: &[ScreenHint] = &[
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::Confirm, "Open"),
    (ActionId::GoHome, "Home"),
];

const SETTINGS_THEMES_HINTS: &[ScreenHint] = &[
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::Confirm, "Apply"),
    (ActionId::GoHome, "Home"),
];

const SETTINGS_THEME_FORM_HINTS: &[ScreenHint] = &[
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::Confirm, "Save"),
    (ActionId::GoHome, "Home"),
];

const CONFLICTS_HINTS: &[ScreenHint] = &[
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::ResolveConflictLocal, "Use local"),
    (ActionId::ResolveConflictRemote, "Use Jira"),
    (ActionId::GoHome, "Home"),
];

const SYNC_STATUS_HINTS: &[ScreenHint] = &[
    (ActionId::SyncNow, "Sync now"),
    (ActionId::SyncPause, "Pause"),
    (ActionId::SyncRetry, "Retry"),
    (ActionId::SyncResume, "Resume"),
    (ActionId::FilterAll, "All"),
    (ActionId::FilterPull, "Pull"),
    (ActionId::FilterPush, "Push"),
    (ActionId::GoHome, "Home"),
];

const FORM_HINTS: &[ScreenHint] = &[
    (ActionId::MoveUp, "Up"),
    (ActionId::MoveDown, "Down"),
    (ActionId::GoHome, "Home"),
];

pub(super) fn screen_hint_actions(screen: ScreenType) -> &'static [ScreenHint] {
    match screen {
        ScreenType::Home => HOME_HINTS,
        ScreenType::CurrentSprint => CURRENT_SPRINT_HINTS,
        ScreenType::IssueDetail => ISSUE_DETAIL_HINTS,
        ScreenType::MyIssues | ScreenType::SearchIssues => ISSUES_WORKSPACE_HINTS,
        ScreenType::BoardSelection => BOARD_SELECTION_HINTS,
        ScreenType::Profiles => PROFILES_HINTS,
        ScreenType::Settings => SETTINGS_HINTS,
        ScreenType::SettingsThemes => SETTINGS_THEMES_HINTS,
        ScreenType::SettingsThemeForm => SETTINGS_THEME_FORM_HINTS,
        ScreenType::Conflicts => CONFLICTS_HINTS,
        ScreenType::SyncStatus => SYNC_STATUS_HINTS,
        ScreenType::NewIssue | ScreenType::ProfileCreation => FORM_HINTS,
    }
}
