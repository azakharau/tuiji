use crossterm::event::KeyEvent;

use std::sync::Arc;

use crate::{config::KeyBindings, ui::screens::ScreenState};

pub mod navigation_hanler;

pub trait KeyHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent, bindings: &KeyBindings) -> ScreenState;
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
