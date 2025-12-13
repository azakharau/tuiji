use crossterm::event::KeyEvent;

use crate::ui::screens::ScreenState;

pub mod navigation_hanler;

pub trait KeyHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ScreenState;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionItem {
    pub binding: &'static str,
    pub description: &'static str,
}

impl ActionItem {
    pub fn render(&self) -> String {
        format!("[{}]  {}", self.binding, self.description)
    }
}
