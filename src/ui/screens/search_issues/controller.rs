use crossterm::event::KeyCode;
use tui_input::InputRequest;

use crate::ui::{
    interaction::{ActionId, Command, Mode},
    screens::{ScreenState, issues_table::IssuesTableController},
};

use super::state::SearchIssuesState;

pub struct SearchIssuesController;

impl SearchIssuesController {
    pub fn handle_command(
        state: &mut SearchIssuesState,
        command: Command,
        mode: Mode,
    ) -> ScreenState {
        if mode == Mode::Insert {
            return handle_query_input(state, command.action);
        }

        match command.action {
            ActionId::FocusQuery => ScreenState::SwitchMode(Mode::Insert),
            ActionId::Confirm => state
                .table()
                .selected_issue()
                .map(|issue| ScreenState::ViewIssue(issue.key.clone()))
                .unwrap_or(ScreenState::Stay),
            _ => IssuesTableController::handle_command(state.table_mut(), command),
        }
    }
}

fn handle_query_input(state: &mut SearchIssuesState, action: ActionId) -> ScreenState {
    let request = match action {
        ActionId::RawInput(KeyCode::Char(ch)) => Some(InputRequest::InsertChar(ch)),
        ActionId::RawInput(KeyCode::Backspace) => Some(InputRequest::DeletePrevChar),
        ActionId::RawInput(KeyCode::Delete) => Some(InputRequest::DeleteNextChar),
        ActionId::RawInput(KeyCode::Left) => Some(InputRequest::GoToPrevChar),
        ActionId::RawInput(KeyCode::Right) => Some(InputRequest::GoToNextChar),
        ActionId::RawInput(KeyCode::Home) => Some(InputRequest::GoToStart),
        ActionId::RawInput(KeyCode::End) => Some(InputRequest::GoToEnd),
        ActionId::RawInput(KeyCode::Enter) => {
            return state
                .submit_query()
                .map(ScreenState::RunSearch)
                .unwrap_or(ScreenState::Refresh);
        }
        _ => None,
    };

    if let Some(request) = request {
        state.edit(request);
        ScreenState::Refresh
    } else {
        ScreenState::Stay
    }
}
