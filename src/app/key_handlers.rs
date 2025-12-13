use std::sync::Arc;

use crate::{
    app::state::{Mode, ScreenType},
    config::KeyBindings,
    ui::screens::ScreenState,
};
use crossterm::event::KeyEvent;

pub mod navigation_hanler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Motion(Motion),
    Refresh,
    Quit,
    SwitchTo(ScreenType),
    Unhandled(KeyEvent),
    Noop,
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

pub fn global_action_hints(bindings: &KeyBindings) -> Arc<Vec<ActionHint>> {
    Arc::new(vec![
        ActionHint {
            binding: bindings.quit.clone(),
            description: "Quit".to_string(),
        },
        ActionHint {
            binding: bindings.refresh.clone(),
            description: "Refresh".to_string(),
        },
        ActionHint {
            binding: bindings.next.clone(),
            description: "Next".to_string(),
        },
        ActionHint {
            binding: bindings.previous.clone(),
            description: "Previous".to_string(),
        },
        ActionHint {
            binding: bindings.open_in_browser.clone(),
            description: "Open".to_string(),
        },
    ])
}

pub fn binding_matches(key: &KeyEvent, binding: &str) -> bool {
    use crossterm::event::KeyCode;
    if binding.is_empty() {
        return false;
    }
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

#[derive(Default)]
pub struct InputState {
    pending_count: Option<usize>,
    pending_g: bool,
}

pub fn parse_command(
    key_event: KeyEvent,
    mode: crate::app::state::Mode,
    bindings: &KeyBindings,
    state: &mut InputState,
) -> Command {
    use crossterm::event::KeyCode;

    // Refresh
    if binding_matches(&key_event, &bindings.refresh) {
        state.pending_count = None;
        state.pending_g = false;
        return Command::Refresh;
    }

    // Quit (only in Normal)
    if (key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Char('Q'))
        && matches!(mode, Mode::Normal)
    {
        state.pending_count = None;
        state.pending_g = false;
        return Command::Quit;
    }

    // Esc -> Home
    if key_event.code == KeyCode::Esc {
        state.pending_count = None;
        state.pending_g = false;
        return Command::SwitchTo(ScreenType::Home);
    }

    // In Insert/Command modes we don't interpret motions or counts — pass raw input through.
    if matches!(mode, Mode::Insert | Mode::Command) {
        state.pending_count = None;
        state.pending_g = false;
        return Command::Unhandled(key_event);
    }

    // Counts
    if let KeyCode::Char(d @ '0'..='9') = key_event.code {
        let digit = d.to_digit(10).unwrap() as usize;
        let new_count = state
            .pending_count
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(digit);
        state.pending_count = Some(new_count);
        return Command::Noop;
    }

    // gg / g
    if key_event.code == KeyCode::Char('g') {
        if state.pending_g {
            state.pending_g = false;
            let _ = take_count(state);
            return Command::Motion(Motion::Top);
        } else {
            state.pending_g = true;
            return Command::Noop;
        }
    }
    state.pending_g = false;

    // Motions
    let motion = match key_event.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Motion::Down(take_count_or(state, 1))),
        KeyCode::Char('k') | KeyCode::Up => Some(Motion::Up(take_count_or(state, 1))),
        KeyCode::Char('h') | KeyCode::Left => Some(Motion::Left(take_count_or(state, 1))),
        KeyCode::Char('l') | KeyCode::Right => Some(Motion::Right(take_count_or(state, 1))),
        KeyCode::Char('G') => Some(Motion::Bottom),
        _ => None,
    };

    if let Some(m) = motion {
        return Command::Motion(m);
    }

    // Default: pass raw key to screen-level handler
    state.pending_count = None;
    Command::Unhandled(key_event)
}

fn take_count(state: &mut InputState) -> usize {
    let n = state.pending_count.take().unwrap_or(1);
    if n == 0 { 1 } else { n }
}

fn take_count_or(state: &mut InputState, default: usize) -> usize {
    let n = state.pending_count.take().unwrap_or(default);
    if n == 0 { default } else { n }
}
