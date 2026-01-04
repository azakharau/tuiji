use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{key_handlers::KeyBinding, state::Mode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedInput {
    Binding { binding: String, repeat: usize },
    ModeSwitch(Mode),
    ToggleHints,
    Text(TextInput),
}

#[derive(Default, Debug, Clone)]
pub struct InputParser {
    pending_count: Option<usize>,
    pending_prefix: Option<char>,
}

impl InputParser {
    pub fn parse(
        &mut self,
        key: KeyEvent,
        mode: Mode,
        bindings: &[KeyBinding],
    ) -> Option<ParsedInput> {
        if mode == Mode::Normal && matches!(key.code, KeyCode::Char('?')) {
            self.reset_state();
            return Some(ParsedInput::ToggleHints);
        }
        match mode {
            Mode::Insert | Mode::Command => self.parse_text_mode(key),
            Mode::Normal | Mode::Visual => self.parse_action_mode(key, bindings),
        }
    }

    pub fn pending_prefix(&self) -> Option<String> {
        self.pending_prefix.map(|c| c.to_string())
    }

    pub fn clear_pending(&mut self) {
        self.reset_state();
    }

    fn parse_text_mode(&mut self, key: KeyEvent) -> Option<ParsedInput> {
        self.reset_state();
        if has_ctrl_or_alt(&key) {
            return None;
        }

        match key.code {
            KeyCode::Char(c) => Some(ParsedInput::Text(TextInput::Char(c))),
            KeyCode::Backspace => Some(ParsedInput::Text(TextInput::Backspace)),
            KeyCode::Delete => Some(ParsedInput::Text(TextInput::Delete)),
            KeyCode::Enter => Some(ParsedInput::Text(TextInput::Enter)),
            KeyCode::Tab => Some(ParsedInput::Text(TextInput::Tab)),
            KeyCode::Esc => Some(ParsedInput::Text(TextInput::Esc)),
            _ => None,
        }
    }

    fn parse_action_mode(&mut self, key: KeyEvent, bindings: &[KeyBinding]) -> Option<ParsedInput> {
        if has_ctrl_or_alt(&key) {
            self.reset_state();
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                self.reset_state();
                return Some(ParsedInput::ModeSwitch(Mode::Normal));
            }
            KeyCode::Char('v') => {
                self.reset_state();
                return Some(ParsedInput::ModeSwitch(Mode::Visual));
            }
            KeyCode::Char(':') => {
                self.reset_state();
                return Some(ParsedInput::ModeSwitch(Mode::Command));
            }
            KeyCode::Char(';') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.reset_state();
                return Some(ParsedInput::ModeSwitch(Mode::Command));
            }
            _ => {}
        }

        if let KeyCode::Char(d @ '0'..='9') = key.code {
            if d == '0'
                && self.pending_count.is_none()
                && bindings.iter().any(|entry| entry.binding == "0")
            {
                let repeat = self.take_count_or(1);
                return Some(ParsedInput::Binding {
                    binding: "0".to_string(),
                    repeat,
                });
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
                if bindings.iter().any(|entry| entry.binding == binding) {
                    self.pending_prefix = None;
                    let repeat = self.take_count_or(1);
                    return Some(ParsedInput::Binding { binding, repeat });
                }
            }
            self.reset_state();
            return None;
        }

        if let KeyCode::Char(prefix) = key.code
            && bindings
                .iter()
                .any(|entry| entry.binding.starts_with(prefix) && entry.binding.len() > 1)
        {
            self.pending_prefix = Some(prefix);
            return None;
        }

        for entry in bindings {
            if entry.binding.len() > 1 && !entry.binding.starts_with('<') {
                continue;
            }
            if binding_matches(&key, &entry.binding) {
                let repeat = self.take_count_or(1);
                return Some(ParsedInput::Binding {
                    binding: entry.binding.clone(),
                    repeat,
                });
            }
        }

        self.reset_state();
        None
    }

    fn reset_state(&mut self) {
        self.pending_prefix = None;
        self.pending_count = None;
    }

    fn take_count_or(&mut self, default: usize) -> usize {
        let n = self.pending_count.take().unwrap_or(default);
        if n == 0 { default } else { n }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInput {
    Char(char),
    Backspace,
    Delete,
    Enter,
    Tab,
    Esc,
}

fn binding_matches(key: &KeyEvent, binding: &str) -> bool {
    match key.code {
        KeyCode::Char(c) => binding.len() == 1 && binding.starts_with(c),
        KeyCode::Enter => binding.eq_ignore_ascii_case("enter") || binding == "<enter>",
        KeyCode::Up => binding.eq_ignore_ascii_case("up") || binding == "<up>",
        KeyCode::Down => binding.eq_ignore_ascii_case("down") || binding == "<down>",
        KeyCode::Left => binding.eq_ignore_ascii_case("left") || binding == "<left>",
        KeyCode::Right => binding.eq_ignore_ascii_case("right") || binding == "<right>",
        KeyCode::Tab => binding.eq_ignore_ascii_case("tab") || binding == "<tab>",
        _ => false,
    }
}

fn has_ctrl_or_alt(key: &KeyEvent) -> bool {
    let mods = key.modifiers;
    mods.contains(KeyModifiers::CONTROL) || mods.contains(KeyModifiers::ALT)
}
