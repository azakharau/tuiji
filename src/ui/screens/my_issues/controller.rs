use crate::ui::{
    interaction::{ActionId, Command},
    screens::{ScreenState, issues_table::IssuesTableController},
};

use super::state::MyIssuesState;

pub struct MyIssuesController;

impl MyIssuesController {
    pub fn handle_command(state: &mut MyIssuesState, command: Command) -> ScreenState {
        if command.action == ActionId::Confirm {
            return state
                .table()
                .selected_issue()
                .map(|issue| ScreenState::ViewIssue(issue.key.clone()))
                .unwrap_or(ScreenState::Stay);
        }

        IssuesTableController::handle_command(state.table_mut(), command)
    }
}
