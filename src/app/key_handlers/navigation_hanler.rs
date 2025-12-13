use crate::{app::key_handlers::{Command, KeyHandler}, ui::screens::ScreenState};

pub struct VimRowNavigationHandler;

impl KeyHandler for VimRowNavigationHandler {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command {
            Command::Motion(_) => ScreenState::Refresh,
            _ => ScreenState::Stay,
        }
    }
}
