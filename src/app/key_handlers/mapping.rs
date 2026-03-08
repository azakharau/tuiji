use super::*;

pub(super) fn map_bindings(entries: &[crate::config::KeyBindingConfig]) -> Vec<KeyBinding> {
    entries
        .iter()
        .map(|entry| KeyBinding {
            action: binding_action_to_action_id(entry.action),
            binding: entry.binding.clone(),
        })
        .collect()
}

pub(super) fn merge_bindings(global: &[KeyBinding], local: &[KeyBinding]) -> Vec<KeyBinding> {
    let mut merged = Vec::with_capacity(global.len() + local.len());
    merged.extend_from_slice(global);
    merged.extend_from_slice(local);
    merged
}

pub(super) fn binding_action_to_action_id(action: BindingAction) -> ActionId {
    match action {
        BindingAction::Quit => ActionId::Quit,
        BindingAction::Refresh => ActionId::Refresh,
        BindingAction::Confirm => ActionId::Confirm,
        BindingAction::GoHome => ActionId::GoHome,
        BindingAction::OpenCurrentSprint => ActionId::OpenCurrentSprint,
        BindingAction::OpenMyIssues => ActionId::OpenMyIssues,
        BindingAction::OpenSearchIssues => ActionId::OpenSearchIssues,
        BindingAction::OpenNewIssue => ActionId::OpenNewIssue,
        BindingAction::OpenProfiles => ActionId::OpenProfiles,
        BindingAction::OpenBoards => ActionId::OpenBoards,
        BindingAction::OpenSettings => ActionId::OpenSettings,
        BindingAction::OpenSyncStatus => ActionId::OpenSyncStatus,
        BindingAction::ResolveConflictLocal => ActionId::ResolveConflictLocal,
        BindingAction::ResolveConflictRemote => ActionId::ResolveConflictRemote,
        BindingAction::SyncNow => ActionId::SyncNow,
        BindingAction::SyncPause => ActionId::SyncPause,
        BindingAction::SyncRetry => ActionId::SyncRetry,
        BindingAction::SyncResume => ActionId::SyncResume,
        BindingAction::FilterAll => ActionId::FilterAll,
        BindingAction::FilterPull => ActionId::FilterPull,
        BindingAction::FilterPush => ActionId::FilterPush,
        BindingAction::NewProfile => ActionId::NewProfile,
        BindingAction::EditProfile => ActionId::EditProfile,
        BindingAction::DeleteProfile => ActionId::DeleteProfile,
        BindingAction::OpenInBrowser => ActionId::OpenInBrowser,
        BindingAction::MoveUp => ActionId::MoveUp,
        BindingAction::MoveDown => ActionId::MoveDown,
        BindingAction::MoveLeft => ActionId::MoveLeft,
        BindingAction::MoveRight => ActionId::MoveRight,
        BindingAction::MoveTop => ActionId::MoveTop,
        BindingAction::MoveBottom => ActionId::MoveBottom,
        BindingAction::MoveLineStart => ActionId::MoveLineStart,
        BindingAction::MoveLineEnd => ActionId::MoveLineEnd,
        BindingAction::MoveWordForward => ActionId::MoveWordForward,
        BindingAction::MoveWordBackward => ActionId::MoveWordBackward,
        BindingAction::MoveWordEnd => ActionId::MoveWordEnd,
        BindingAction::EnterInsertBefore => ActionId::EnterInsert(InsertMode::Before),
        BindingAction::EnterInsertAfter => ActionId::EnterInsert(InsertMode::After),
        BindingAction::EnterInsertLineStart => ActionId::EnterInsert(InsertMode::LineStart),
        BindingAction::EnterInsertLineEnd => ActionId::EnterInsert(InsertMode::LineEnd),
    }
}
