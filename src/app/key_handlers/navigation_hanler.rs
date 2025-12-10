use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{KeyHandler, ScreenState};

pub struct VimRowNavigationHandler;

impl KeyHandler for VimRowNavigationHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ScreenState {
        match key_event.code {
            KeyCode::Char('j') => {
                // Handle down movement
                ScreenState::OneRowDown
            }
            KeyCode::Char('k') => {
                // Handle up movement
                ScreenState::OneRowUp
            }
            KeyCode::Char('h') => {
                // Handle left movement
                ScreenState::OneColumnLeft
            }
            KeyCode::Char('l') => {
                // Handle right movement
                ScreenState::OneColumnRight
            }
            KeyCode::Char('d') => {
                // Handle half-page down movement
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    return ScreenState::HalfPageDown;
                }
                ScreenState::Stay
            }
            KeyCode::Char('u') => {
                // Handle half-page up movement
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    return ScreenState::HalfPageUp;
                }
                ScreenState::Stay
            }
            _ => ScreenState::Stay,
        }
    }
}
