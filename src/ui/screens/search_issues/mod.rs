mod controller;
mod state;
mod view;

use std::sync::Arc;

use color_eyre::Result;
use ratatui::Frame;

use crate::{
    app::{
        key_handlers::{ActionHint, Command, KeyHandler},
        state::Mode,
    },
    data::AppRepository,
    ui::{
        context::RenderContext,
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
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let issues = repo.current_sprint_issues(board_id).await?;
        Ok(Self {
            state: SearchIssuesState::new(issues),
            actions: Arc::new(Vec::new()),
            mode,
        })
    }
}

impl Screen for SearchIssuesScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        SearchIssuesView::draw(frame, &self.state, self.mode, &self.actions, context);
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
        SearchIssuesController::handle_command(&mut self.state, command)
    }
}
