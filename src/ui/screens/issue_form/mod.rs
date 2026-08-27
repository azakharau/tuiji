mod controller;
mod state;
mod view;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    app::FormPurpose,
    ui::{
        context::RenderContext,
        interaction::Mode,
        interaction::{ActionHint, Command, KeyHandler},
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::IssueFormController;
use state::IssueFormState;
use view::IssueFormView;

pub struct IssueFormScreen {
    state: IssueFormState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl Default for IssueFormScreen {
    fn default() -> Self {
        Self::new(FormPurpose::Create, None, Vec::new(), String::new(), None)
    }
}

impl IssueFormScreen {
    pub fn new(
        purpose: FormPurpose,
        project_key: Option<String>,
        issue_types: Vec<String>,
        summary: String,
        description: Option<String>,
    ) -> Self {
        let state = match purpose {
            FormPurpose::Create => IssueFormState::create(project_key, issue_types),
            FormPurpose::Edit(key) => IssueFormState::edit(key, summary, description),
        };

        Self {
            state,
            actions: Arc::new(Vec::new()),
            mode: Mode::Normal,
        }
    }

    /// Surfaces a load failure raised while the factory was building the form,
    /// so an empty Issue Type list explains itself instead of failing validation
    /// with "issue type is required".
    pub fn set_error(&mut self, error: crate::contracts::error::AppErrorState) {
        self.state.set_error(error);
    }
}

impl Screen for IssueFormScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        IssueFormView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        self.state.screen_name()
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        IssueFormController::handle_command_line(&mut self.state, cmd)
    }
}

impl KeyHandler for IssueFormScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        IssueFormController::handle_command(&mut self.state, command, self.mode)
    }
}
