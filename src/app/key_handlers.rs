use std::sync::Arc;

use crossterm::event::KeyCode;

use crate::{
    app::state::{Mode, ScreenType},
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
    NextRow,
    SwitchModeTo(Mode),
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

/// Mapping: actions available on a screen and their bindings.
pub fn screen_bindings(screen: ScreenType) -> Vec<(ActionId, String)> {
    let mut map = vec![
        (ActionId::Refresh, "r".to_string()),
        (ActionId::Confirm, "<enter>".to_string()),
        (ActionId::GoHome, "gh".to_string()),
        (ActionId::OpenInBrowser, "o".to_string()),
    ];

    match screen {
        ScreenType::Home => map.extend(home_defaults()),
        ScreenType::CurrentSprint => map.extend(current_sprint_defaults()),
        ScreenType::ProfileCreation => map.extend(form_defaults()),
        ScreenType::Profiles
        | ScreenType::MyIssues
        | ScreenType::SearchIssues
        | ScreenType::NewIssue => {}
    }

    map
}

fn home_defaults() -> Vec<(ActionId, String)> {
    vec![
        (ActionId::Quit, "q".to_string()),
        (ActionId::OpenCurrentSprint, "c".to_string()),
        (ActionId::OpenMyIssues, "i".to_string()),
        (ActionId::OpenSearchIssues, "s".to_string()),
        (ActionId::OpenNewIssue, "n".to_string()),
        (ActionId::OpenProfiles, "p".to_string()),
        (ActionId::MoveUp, "k".to_string()),
        (ActionId::MoveUp, "<up>".to_string()),
        (ActionId::MoveDown, "j".to_string()),
        (ActionId::MoveDown, "<down>".to_string()),
    ]
}

fn current_sprint_defaults() -> Vec<(ActionId, String)> {
    vec![
        (ActionId::MoveUp, "k".to_string()),
        (ActionId::MoveUp, "<up>".to_string()),
        (ActionId::MoveDown, "j".to_string()),
        (ActionId::MoveDown, "<down>".to_string()),
        (ActionId::MoveLeft, "h".to_string()),
        (ActionId::MoveLeft, "<left>".to_string()),
        (ActionId::MoveRight, "l".to_string()),
        (ActionId::MoveRight, "<right>".to_string()),
        (ActionId::MoveTop, "gg".to_string()),
        (ActionId::MoveBottom, "G".to_string()),
    ]
}

fn form_defaults() -> Vec<(ActionId, String)> {
    vec![
        (ActionId::MoveUp, "k".to_string()),
        (ActionId::MoveUp, "<up>".to_string()),
        (ActionId::MoveDown, "j".to_string()),
        (ActionId::MoveDown, "<down>".to_string()),
        (ActionId::MoveLeft, "h".to_string()),
        (ActionId::MoveLeft, "<left>".to_string()),
        (ActionId::MoveRight, "l".to_string()),
        (ActionId::MoveRight, "<right>".to_string()),
        (ActionId::MoveTop, "gg".to_string()),
        (ActionId::MoveBottom, "G".to_string()),
        (ActionId::MoveLineStart, "0".to_string()),
        (ActionId::MoveLineStart, "^".to_string()),
        (ActionId::MoveLineEnd, "$".to_string()),
        (ActionId::MoveWordForward, "w".to_string()),
        (ActionId::MoveWordForward, "W".to_string()),
        (ActionId::MoveWordBackward, "b".to_string()),
        (ActionId::MoveWordBackward, "B".to_string()),
        (ActionId::MoveWordEnd, "e".to_string()),
        (ActionId::MoveWordEnd, "E".to_string()),
        (ActionId::EnterInsert(InsertMode::Before), "i".to_string()),
        (ActionId::EnterInsert(InsertMode::After), "a".to_string()),
        (ActionId::EnterInsert(InsertMode::LineStart), "I".to_string()),
        (ActionId::EnterInsert(InsertMode::LineEnd), "A".to_string()),
    ]
}

/// Generates bottom-bar hints from the current bindings.
pub fn action_hints(screen: ScreenType) -> Arc<Vec<ActionHint>> {
    let mut hints = Vec::new();
    let bindings = screen_bindings(screen);
    let first = |id: ActionId| {
        bindings
            .iter()
            .find(|(a, _)| *a == id)
            .map(|(_, b)| b.clone())
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
        _ => {}
    }

    Arc::new(hints)
}

pub fn binding_hints_for_prefix(screen: ScreenType, prefix: &str) -> Vec<ActionHint> {
    let mut hints = Vec::new();
    for (action, binding) in screen_bindings(screen) {
        if !binding.starts_with(prefix) || binding.len() == prefix.len() {
            continue;
        }
        if let Some(description) = action_description(action) {
            hints.push(ActionHint {
                binding,
                description: description.to_string(),
            });
        }
    }
    hints.sort_by(|a, b| a.binding.cmp(&b.binding));
    hints
}

pub fn binding_hints_for_screen(screen: ScreenType) -> Vec<ActionHint> {
    let mut hints = Vec::new();
    for (action, binding) in screen_bindings(screen) {
        if let Some(description) = action_description(action) {
            hints.push(ActionHint {
                binding,
                description: description.to_string(),
            });
        }
    }
    hints.sort_by(|a, b| a.binding.cmp(&b.binding));
    hints
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
        ActionId::NextRow => Some("Next row"),
        ActionId::SwitchModeTo(_) => Some("Switch mode"),
        ActionId::EnterInsert(_) => Some("Insert"),
        ActionId::RawInput(_) => None,
    }
}
