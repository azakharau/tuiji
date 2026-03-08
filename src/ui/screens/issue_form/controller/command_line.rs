use crate::{
    contracts::error::AppErrorState,
    ui::screens::{CommandLineCommand, ScreenState},
};

use super::IssueFormController;
use crate::ui::screens::issue_form::state::IssueFormState;

impl IssueFormController {
    pub fn handle_command_line(state: &mut IssueFormState, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write | CommandLineCommand::WriteQuit => match validate_form(state)
            {
                Ok(()) => ScreenState::Close,
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

fn validate_form(state: &IssueFormState) -> Result<(), String> {
    let mut errors = Vec::new();

    for field in state.form().fields() {
        if let Some(error) = field.validate() {
            errors.push(error.message);
        }
    }

    if !errors.is_empty() {
        return Err(errors.join(", "));
    }

    let summary = state
        .form()
        .fields()
        .first()
        .and_then(|field| field.value.as_text())
        .unwrap_or("");

    if summary.trim().is_empty() {
        return Err("Summary is required".to_string());
    }

    Ok(())
}
