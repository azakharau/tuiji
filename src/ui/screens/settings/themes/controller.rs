use crate::{
    ui::interaction::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::SettingsThemesState;

pub struct SettingsThemesController;

impl SettingsThemesController {
    pub fn handle_command(state: &mut SettingsThemesState, command: Command) -> ScreenState {
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
            ActionId::Confirm => {
                if state.selected_is_create() {
                    ScreenState::SwitchTo(crate::ui::interaction::ScreenType::SettingsThemeForm)
                } else {
                    state
                        .selected_theme_id()
                        .map(|id| ScreenState::ApplyTheme(id.to_string()))
                        .unwrap_or(ScreenState::Stay)
                }
            }
            _ => ScreenState::Stay,
        }
    }
}
