use crate::{
    app::key_handlers::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::ProfilesState;

pub struct ProfilesController;

impl ProfilesController {
    pub fn handle_command(state: &mut ProfilesState, command: Command) -> ScreenState {
        match command.action {
            ActionId::MoveUp => {
                state.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                state.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.move_bottom();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}
