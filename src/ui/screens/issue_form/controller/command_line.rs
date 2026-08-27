use crate::{
    contracts::error::AppErrorState,
    ui::screens::{CommandLineCommand, ScreenState},
};

use super::IssueFormController;
use crate::ui::screens::issue_form::state::IssueFormState;

impl IssueFormController {
    pub fn handle_command_line(state: &mut IssueFormState, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            command @ (CommandLineCommand::Write | CommandLineCommand::WriteQuit) => {
                let close_when_unchanged = matches!(command, CommandLineCommand::WriteQuit);
                state.clear_error();

                match state.submission() {
                    Ok(Some(mutation)) => ScreenState::Mutate(mutation),
                    Ok(_) if close_when_unchanged => ScreenState::Close,
                    Ok(_) => ScreenState::Refresh,
                    Err(err) => {
                        state.set_error(AppErrorState::new("Validation Error", err));
                        ScreenState::Refresh
                    }
                }
            }
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}
