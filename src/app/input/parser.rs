use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{
    key_handlers::{ActionId, Command, screen_bindings},
    state::{Mode, ScreenType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Action(Command),
    ModeSwitch(Mode),
    ToggleHints,
    Text(char),
    Backspace,
    Delete,
    Enter,
    Tab,
    Esc,
}

#[derive(Default, Debug, Clone)]
pub struct InputParser {
    pending_count: Option<usize>,
    pending_prefix: Option<char>,
}

impl InputParser {
    pub fn parse(&mut self, key: KeyEvent, mode: Mode, screen: ScreenType) -> Option<InputEvent> {
        if mode == Mode::Normal && matches!(key.code, KeyCode::Char('?')) {
            self.reset_state();
            return Some(InputEvent::ToggleHints);
        }
        match mode {
            Mode::Insert | Mode::Command => self.parse_text_mode(key),
            Mode::Normal | Mode::Visual => self.parse_action_mode(key, screen),
        }
    }

    pub fn pending_prefix(&self) -> Option<String> {
        self.pending_prefix.map(|c| c.to_string())
    }

    pub fn clear_pending(&mut self) {
        self.reset_state();
    }

    fn parse_text_mode(&mut self, key: KeyEvent) -> Option<InputEvent> {
        self.reset_state();
        if has_ctrl_or_alt(&key) {
            return None;
        }

        match key.code {
            KeyCode::Char(c) => Some(InputEvent::Text(c)),
            KeyCode::Backspace => Some(InputEvent::Backspace),
            KeyCode::Delete => Some(InputEvent::Delete),
            KeyCode::Enter => Some(InputEvent::Enter),
            KeyCode::Tab => Some(InputEvent::Tab),
            KeyCode::Esc => Some(InputEvent::Esc),
            _ => None,
        }
    }

    fn parse_action_mode(&mut self, key: KeyEvent, screen: ScreenType) -> Option<InputEvent> {
        if has_ctrl_or_alt(&key) {
            self.reset_state();
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                self.reset_state();
                return Some(InputEvent::ModeSwitch(Mode::Normal));
            }
            KeyCode::Char('v') => {
                self.reset_state();
                return Some(InputEvent::ModeSwitch(Mode::Visual));
            }
            KeyCode::Char(':') => {
                self.reset_state();
                return Some(InputEvent::ModeSwitch(Mode::Command));
            }
            KeyCode::Char(';') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.reset_state();
                return Some(InputEvent::ModeSwitch(Mode::Command));
            }
            _ => {}
        }

        if let KeyCode::Char(d @ '0'..='9') = key.code {
            if d == '0'
                && self.pending_count.is_none()
                && let Some((action, _)) =
                    screen_bindings(screen).into_iter().find(|(_, b)| b == "0")
            {
                let repeat = if is_motion(action) {
                    self.take_count_or(1)
                } else {
                    self.reset_count();
                    1
                };
                return Some(InputEvent::Action(Command { action, repeat }));
            }
            let digit = d.to_digit(10).unwrap_or(0) as usize;
            let new_count = self
                .pending_count
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add(digit);
            self.pending_count = Some(new_count);
            return None;
        }

        if let Some(prefix) = self.pending_prefix {
            if let KeyCode::Char(next) = key.code {
                let binding = format!("{}{}", prefix, next);
                if let Some((action, _)) = screen_bindings(screen)
                    .into_iter()
                    .find(|(_, b)| b == &binding)
                {
                    self.pending_prefix = None;
                    let repeat = if is_motion(action) {
                        self.take_count_or(1)
                    } else {
                        self.reset_count();
                        1
                    };
                    return Some(InputEvent::Action(Command { action, repeat }));
                }
            }
            self.reset_state();
            return None;
        }

        if let KeyCode::Char(prefix) = key.code
            && screen_bindings(screen)
                .iter()
                .any(|(_, key)| key.starts_with(prefix) && key.len() > 1)
        {
            self.pending_prefix = Some(prefix);
            return None;
        }

        let candidates = screen_bindings(screen);
        for (action, binding) in candidates {
            if binding.len() > 1 && !binding.starts_with('<') {
                continue;
            }
            if binding_matches(&key, &binding) {
                let repeat = if is_motion(action) {
                    self.take_count_or(1)
                } else {
                    self.reset_count();
                    1
                };
                return Some(InputEvent::Action(Command { action, repeat }));
            }
        }

        self.reset_state();
        None
    }

    fn reset_state(&mut self) {
        self.pending_prefix = None;
        self.pending_count = None;
    }

    fn reset_count(&mut self) {
        self.pending_count = None;
    }

    fn take_count_or(&mut self, default: usize) -> usize {
        let n = self.pending_count.take().unwrap_or(default);
        if n == 0 { default } else { n }
    }
}

pub fn is_question_mark(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('?'))
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
        KeyCode::Tab => binding.eq_ignore_ascii_case("tab") || binding == "<tab>",
        _ => false,
    }
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
            | ActionId::MoveLineStart
            | ActionId::MoveLineEnd
            | ActionId::MoveWordForward
            | ActionId::MoveWordBackward
            | ActionId::MoveWordEnd
    )
}

fn has_ctrl_or_alt(key: &KeyEvent) -> bool {
    let mods = key.modifiers;
    mods.contains(KeyModifiers::CONTROL) || mods.contains(KeyModifiers::ALT)
}
