use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};

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
    NextRow,
    RawInput(KeyCode),
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

#[derive(Default)]
pub struct InputState {
    pending_count: Option<usize>,
    pending_g: bool,
}

/// Main parser: maps a key event to an ActionId for the active screen.
pub fn parse_command(
    key_event: KeyEvent,
    mode: Mode,
    state: &mut InputState,
    screen: ScreenType,
) -> Option<Command> {
    // In Insert/Command modes we don't intercept yet — can be extended later.
    if matches!(mode, Mode::Insert | Mode::Command) {
        reset_state(state);
        return None;
    }

    // Numeric prefixes (e.g. 3j)
    if let KeyCode::Char(d @ '0'..='9') = key_event.code {
        let digit = d.to_digit(10).unwrap_or(0) as usize;
        let new_count = state
            .pending_count
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(digit);
        state.pending_count = Some(new_count);
        return None;
    }

    // gg is a special case
    if key_event.code == KeyCode::Char('g') {
        if state.pending_g
            && screen_bindings(screen)
                .iter()
                .any(|(action, key)| *action == ActionId::MoveTop && key == "gg")
        {
            state.pending_g = false;
            let repeat = take_count_or(state, 1);
            return Some(Command {
                action: ActionId::MoveTop,
                repeat,
            });
        } else {
            state.pending_g = true;
            return None;
        }
    } else {
        state.pending_g = false;
    }

    let candidates = screen_bindings(screen);
    for (action, binding) in candidates {
        if binding == "gg" {
            continue;
        }
        if binding_matches(&key_event, &binding) {
            let repeat = if is_motion(action) {
                take_count_or(state, 1)
            } else {
                reset_count(state);
                1
            };
            return Some(Command { action, repeat });
        }
    }

    reset_state(state);
    None
}

fn binding_matches(key: &KeyEvent, binding: &str) -> bool {
    match key.code {
        KeyCode::Char(c) => binding.len() == 1 && binding.starts_with(c),
        KeyCode::Enter => binding.eq_ignore_ascii_case("enter") || binding == "<enter>",
        KeyCode::Esc => binding.eq_ignore_ascii_case("esc") || binding == "<esc>",
        KeyCode::Up => binding.eq_ignore_ascii_case("up") || binding == "<up>",
        KeyCode::Down => binding.eq_ignore_ascii_case("down") || binding == "<down>",
        KeyCode::Left => binding.eq_ignore_ascii_case("left") || binding == "<left>",
        KeyCode::Right => binding.eq_ignore_ascii_case("right") || binding == "<right>",
        _ => false,
    }
}

fn reset_state(state: &mut InputState) {
    state.pending_g = false;
    state.pending_count = None;
}

fn reset_count(state: &mut InputState) {
    state.pending_count = None;
}

fn take_count_or(state: &mut InputState, default: usize) -> usize {
    let n = state.pending_count.take().unwrap_or(default);
    if n == 0 { default } else { n }
}

fn is_motion(action: ActionId) -> bool {
    matches!(
        action,
        ActionId::MoveUp
            | ActionId::MoveDown
            | ActionId::MoveLeft
            | ActionId::MoveRight
            | ActionId::MoveTop
            | ActionId::MoveBottom
    )
}

/// Mapping: actions available on a screen and their bindings.
pub fn screen_bindings(screen: ScreenType) -> Vec<(ActionId, String)> {
    let mut map = vec![
        (ActionId::Quit, "q".to_string()),
        (ActionId::Refresh, "r".to_string()),
        (ActionId::Confirm, "<enter>".to_string()),
        (ActionId::GoHome, "<esc>".to_string()),
        (ActionId::OpenInBrowser, "o".to_string()),
    ];

    match screen {
        ScreenType::Home => map.extend(home_defaults()),
        ScreenType::CurrentSprint => map.extend(current_sprint_defaults()),
        ScreenType::Profiles
        | ScreenType::MyIssues
        | ScreenType::SearchIssues
        | ScreenType::NewIssue
        | ScreenType::ProfileCreation => {}
    }

    map
}

fn home_defaults() -> Vec<(ActionId, String)> {
    vec![
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
    ]
}

/// Generates bottom-bar hints from the current bindings.
pub fn action_hints(screen: ScreenType) -> Arc<Vec<ActionHint>> {
    let mut hints = Vec::new();
    let bindings = screen_bindings(screen.clone());
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

    push(ActionId::Quit, "Quit");
    push(ActionId::Refresh, "Refresh");

    match screen {
        ScreenType::Home => {
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
        }
        _ => {}
    }

    Arc::new(hints)
}
