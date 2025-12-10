use crossterm::event::KeyEvent;

use crate::ui::screens::ScreenState;

pub mod navigation_hanler;

pub trait KeyHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ScreenState;
}
