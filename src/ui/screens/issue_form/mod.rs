mod controller;
mod state;
mod view;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    ui::{
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
    ui::{
        interaction::Mode,
        interaction::{ActionHint, Command, KeyHandler},
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
        Self::new()
    }
}

impl IssueFormScreen {
    pub fn new() -> Self {
        Self {
            state: IssueFormState::new(),
            actions: Arc::new(Vec::new()),
            mode: Mode::Normal,
        }
    }
}

impl Screen for IssueFormScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        IssueFormView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        self.state.title()
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
