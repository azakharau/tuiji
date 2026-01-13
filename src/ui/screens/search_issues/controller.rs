use crate::{
    app::key_handlers::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::SearchIssuesState;

pub struct SearchIssuesController;

impl SearchIssuesController {
    pub fn handle_command(state: &mut SearchIssuesState, command: Command) -> ScreenState {
        match command.action {
            ActionId::Confirm => {
                if let Some(key) = state.selected_issue_key() {
                    ScreenState::ViewIssue(key.to_string())
                } else {
                    ScreenState::Stay
                }
            }
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
