mod controller;
mod state;
mod view;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    app::{
        key_handlers::{ActionHint, Command, KeyHandler},
        state::Mode,
    },
    data::IssueSummary,
    ui::{
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::IssueDetailController;
use state::IssueDetailState;
use view::IssueDetailView;

pub struct IssueDetailScreen {
    state: IssueDetailState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl IssueDetailScreen {
    pub fn new(issue: IssueSummary, mode: Mode) -> Self {
        Self {
            state: IssueDetailState::new(issue),
            actions: Arc::new(vec![]),
            mode,
        }
    }

    pub fn issue_key(&self) -> &str {
        &self.state.issue().key
    }
}

impl Screen for IssueDetailScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        IssueDetailView::draw(frame, &mut self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Issue Detail"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => ScreenState::Stay,
            CommandLineCommand::WriteQuit => ScreenState::Close,
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

impl KeyHandler for IssueDetailScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        IssueDetailController::handle_command(&mut self.state, command)
    }
}
