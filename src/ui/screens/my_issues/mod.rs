mod controller;
mod state;
mod view;

use std::sync::Arc;

use color_eyre::Result;
use ratatui::Frame;

use crate::{
    data::AppRepository,
    ui::{
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
    ui::{
        interaction::Mode,
        interaction::{ActionHint, Command, KeyHandler},
    },
};

use controller::MyIssuesController;
use state::MyIssuesState;
use view::MyIssuesView;

pub struct MyIssuesScreen {
    state: MyIssuesState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl MyIssuesScreen {
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let issues = repo.current_sprint_issues(board_id).await?;
        Ok(Self {
            state: MyIssuesState::my_issues(issues),
            actions: Arc::new(Vec::new()),
            mode,
        })
    }
}

impl Screen for MyIssuesScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        MyIssuesView::draw(frame, &mut self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "My Issues"
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

impl KeyHandler for MyIssuesScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        MyIssuesController::handle_command(&mut self.state, command)
    }
}
