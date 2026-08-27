mod controller;
mod state;
mod view;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    data::IssueSummary,
    ui::{
        context::RenderContext,
        interaction::{ActionHint, Command, KeyHandler, Mode},
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::SearchIssuesController;
use state::SearchIssuesState;
use view::SearchIssuesView;

pub struct SearchIssuesScreen {
    state: SearchIssuesState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl SearchIssuesScreen {
    pub fn new(
        mode: Mode,
        query: String,
        issues: Vec<IssueSummary>,
        error: Option<String>,
    ) -> Self {
        Self {
            state: SearchIssuesState::new(query, issues, error),
            actions: Arc::new(Vec::new()),
            mode,
        }
    }
}

impl Screen for SearchIssuesScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        SearchIssuesView::draw(frame, &mut self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Search Issues"
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

impl KeyHandler for SearchIssuesScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        SearchIssuesController::handle_command(&mut self.state, command, self.mode)
    }
}
