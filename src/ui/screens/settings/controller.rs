use crate::{
    app::key_handlers::{ActionId, Command},
    app::state::ScreenType,
    ui::screens::ScreenState,
};

use super::state::{SettingsItemId, SettingsState};

pub struct SettingsController;

impl SettingsController {
    pub fn handle_command(state: &mut SettingsState, command: Command) -> ScreenState {
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
            ActionId::Confirm => match state.selected_item() {
                Some(SettingsItemId::Profiles) => ScreenState::SwitchTo(ScreenType::Profiles),
                Some(SettingsItemId::Themes) => ScreenState::SwitchTo(ScreenType::SettingsThemes),
                None => ScreenState::Stay,
            },
            _ => ScreenState::Stay,
        }
    }
}
