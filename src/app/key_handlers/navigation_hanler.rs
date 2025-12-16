use crate::{
    app::key_handlers::{ActionId, Command, KeyHandler},
    ui::screens::ScreenState,
};

pub struct VimRowNavigationHandler;

impl KeyHandler for VimRowNavigationHandler {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::MoveUp
            | ActionId::MoveDown
            | ActionId::MoveLeft
            | ActionId::MoveRight
            | ActionId::MoveTop
            | ActionId::MoveBottom => ScreenState::Refresh,
            _ => ScreenState::Stay,
        }
    }
}
