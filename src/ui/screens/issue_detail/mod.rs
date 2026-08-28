mod controller;
mod state;
mod view;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    data::{IssueSummary, TransitionOptions},
    ui::{
        context::RenderContext,
        interaction::{ActionHint, Command, KeyHandler, Mode},
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
    pub fn new(
        issue: IssueSummary,
        mode: Mode,
        base_url: Option<String>,
        transition_result: Option<Result<TransitionOptions, String>>,
    ) -> Self {
        Self {
            state: IssueDetailState::new(issue, base_url, transition_result),
            actions: Arc::new(vec![]),
            mode,
        }
    }

    pub fn unavailable(message: String, mode: Mode) -> Self {
        Self {
            state: IssueDetailState::unavailable(message),
            actions: Arc::new(vec![]),
            mode,
        }
    }

    pub fn transitions_requested(&self) -> bool {
        self.state.transitions_requested()
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
