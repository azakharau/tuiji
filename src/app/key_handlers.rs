use std::{collections::HashMap, sync::Arc};

use crossterm::event::KeyCode;

use crate::{
    app::state::ScreenType,
    config::{BindingAction, KeyBindingsConfig},
    ui::screens::ScreenState,
};

/// Universal action identifiers; each screen declares which ones it supports.
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
    NewProfile,
    EditProfile,
    DeleteProfile,
    OpenInBrowser,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    pub action: ActionId,
    pub binding: String,
}

#[derive(Clone, Debug)]
pub struct KeyBindings {
    by_screen: HashMap<ScreenType, Arc<Vec<KeyBinding>>>,
}

impl KeyBindings {
    pub fn from_config(cfg: &KeyBindingsConfig) -> Self {
        let global = map_bindings(&cfg.global);
        let mut by_screen = HashMap::new();
        by_screen.insert(
            ScreenType::Home,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.home))),
        );
        by_screen.insert(
            ScreenType::BoardSelection,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.board_selection))),
        );
        by_screen.insert(
            ScreenType::CurrentSprint,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.current_sprint))),
        );
        by_screen.insert(
            ScreenType::ProfileCreation,
            Arc::new(merge_bindings(
                &global,
                &map_bindings(&cfg.profile_creation),
            )),
        );
        by_screen.insert(
            ScreenType::Profiles,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.profiles))),
        );
        by_screen.insert(
            ScreenType::MyIssues,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.my_issues))),
        );
        by_screen.insert(
            ScreenType::SearchIssues,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.search_issues))),
        );
        by_screen.insert(
            ScreenType::NewIssue,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.new_issue))),
        );
        Self { by_screen }
    }

    pub fn bindings_for_screen(&self, screen: ScreenType) -> Arc<Vec<KeyBinding>> {
        self.by_screen
            .get(&screen)
            .cloned()
            .unwrap_or_else(|| Arc::new(Vec::new()))
    }

    pub fn bindings_for_screen_ref(&self, screen: ScreenType) -> &[KeyBinding] {
        self.by_screen
            .get(&screen)
            .map(|bindings| bindings.as_slice())
            .unwrap_or(&[])
    }

    pub fn action_for_binding(&self, screen: ScreenType, binding: &str) -> Option<ActionId> {
        self.by_screen.get(&screen).and_then(|bindings| {
            bindings
                .iter()
                .find(|entry| entry.binding == binding)
                .map(|entry| entry.action)
        })
    }

    pub fn binding_strings_for_screen(&self, screen: ScreenType) -> Vec<String> {
        self.by_screen
            .get(&screen)
            .map(|bindings| bindings.iter().map(|b| b.binding.clone()).collect())
            .unwrap_or_default()
    }
}

/// Generates bottom-bar hints from the current bindings.
pub fn action_hints(screen: ScreenType, bindings: &KeyBindings) -> Arc<Vec<ActionHint>> {
    let mut hints = Vec::new();
    let bindings = bindings.bindings_for_screen(screen);
    let first = |id: ActionId| {
        bindings
            .iter()
            .find(|entry| entry.action == id)
            .map(|entry| entry.binding.clone())
    };

    let mut push = |id: ActionId, description: &str| {
        if let Some(b) = first(id) {
            hints.push(ActionHint {
                binding: b,
                description: description.to_string(),
            });
        }
    };

    push(ActionId::Refresh, "Refresh");

    match screen {
        ScreenType::Home => {
            push(ActionId::Quit, "Quit");
            push(ActionId::Confirm, "Select");
            push(ActionId::MoveUp, "Up");
            push(ActionId::MoveDown, "Down");
            push(ActionId::OpenCurrentSprint, "Current sprint");
            push(ActionId::OpenMyIssues, "My issues");
            push(ActionId::OpenSearchIssues, "Search issues");
            push(ActionId::OpenNewIssue, "New issue");
            push(ActionId::OpenBoards, "Boards");
            push(ActionId::OpenProfiles, "Profiles");
        }
        ScreenType::CurrentSprint => {
            push(ActionId::MoveUp, "Up");
            push(ActionId::MoveDown, "Down");
            push(ActionId::MoveLeft, "Prev column");
            push(ActionId::MoveRight, "Next column");
            push(ActionId::MoveTop, "Top");
            push(ActionId::MoveBottom, "Bottom");
            push(ActionId::GoHome, "Home");
        }
        ScreenType::BoardSelection => {
            push(ActionId::Quit, "Quit");
            push(ActionId::GoHome, "Home");
        }
        ScreenType::Profiles => {
            push(ActionId::MoveUp, "Up");
            push(ActionId::MoveDown, "Down");
            push(ActionId::Confirm, "Activate");
            push(ActionId::EditProfile, "Edit");
            push(ActionId::DeleteProfile, "Delete");
            push(ActionId::NewProfile, "New");
            push(ActionId::GoHome, "Home");
        }
        _ => {}
    }

    Arc::new(hints)
}

fn action_description(action: ActionId) -> Option<&'static str> {
    match action {
        ActionId::Quit => Some("Quit"),
        ActionId::Refresh => Some("Refresh"),
        ActionId::Confirm => Some("Confirm"),
        ActionId::GoHome => Some("Home"),
        ActionId::OpenCurrentSprint => Some("Current sprint"),
        ActionId::OpenMyIssues => Some("My issues"),
        ActionId::OpenSearchIssues => Some("Search issues"),
        ActionId::OpenNewIssue => Some("New issue"),
        ActionId::OpenProfiles => Some("Profiles"),
        ActionId::OpenBoards => Some("Boards"),
        ActionId::NewProfile => Some("New profile"),
        ActionId::EditProfile => Some("Edit profile"),
        ActionId::DeleteProfile => Some("Delete profile"),
        ActionId::OpenInBrowser => Some("Open in browser"),
        ActionId::MoveUp => Some("Up"),
        ActionId::MoveDown => Some("Down"),
        ActionId::MoveLeft => Some("Left"),
        ActionId::MoveRight => Some("Right"),
        ActionId::MoveTop => Some("Top"),
        ActionId::MoveBottom => Some("Bottom"),
        ActionId::MoveLineStart => Some("Line start"),
        ActionId::MoveLineEnd => Some("Line end"),
        ActionId::MoveWordForward => Some("Word forward"),
        ActionId::MoveWordBackward => Some("Word back"),
        ActionId::MoveWordEnd => Some("Word end"),
        ActionId::EnterInsert(_) => Some("Insert"),
        ActionId::RawInput(_) => None,
    }
}

pub fn binding_hints_for_prefix(
    screen: ScreenType,
    prefix: char,
    bindings: &KeyBindings,
) -> Vec<ActionHint> {
    let mut hints = Vec::new();
    for entry in bindings.bindings_for_screen(screen).iter() {
        if !entry.binding.starts_with(prefix) || entry.binding.chars().nth(1).is_none() {
            continue;
        }
        if let Some(description) = action_description(entry.action) {
            hints.push(ActionHint {
                binding: entry.binding.clone(),
                description: description.to_string(),
            });
        }
    }
    hints.sort_by(|a, b| a.binding.cmp(&b.binding));
    hints
}

pub fn binding_hints_for_screen(screen: ScreenType, bindings: &KeyBindings) -> Vec<ActionHint> {
    let mut hints = Vec::new();
    for entry in bindings.bindings_for_screen(screen).iter() {
        if let Some(description) = action_description(entry.action) {
            hints.push(ActionHint {
                binding: entry.binding.clone(),
                description: description.to_string(),
            });
        }
    }
    hints.sort_by(|a, b| a.binding.cmp(&b.binding));
    hints
}

pub fn is_motion_action(action: ActionId) -> bool {
    matches!(
        action,
        ActionId::MoveUp
            | ActionId::MoveDown
            | ActionId::MoveLeft
            | ActionId::MoveRight
            | ActionId::MoveTop
            | ActionId::MoveBottom
            | ActionId::MoveLineStart
            | ActionId::MoveLineEnd
            | ActionId::MoveWordForward
            | ActionId::MoveWordBackward
            | ActionId::MoveWordEnd
    )
}

fn map_bindings(entries: &[crate::config::KeyBindingConfig]) -> Vec<KeyBinding> {
    entries
        .iter()
        .map(|entry| KeyBinding {
            action: binding_action_to_action_id(entry.action),
            binding: entry.binding.clone(),
        })
        .collect()
}

fn merge_bindings(global: &[KeyBinding], local: &[KeyBinding]) -> Vec<KeyBinding> {
    let mut merged = Vec::with_capacity(global.len() + local.len());
    merged.extend_from_slice(global);
    merged.extend_from_slice(local);
    merged
}

fn binding_action_to_action_id(action: BindingAction) -> ActionId {
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
