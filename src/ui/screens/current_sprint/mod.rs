mod controller;
mod detail;
mod formatting;
mod state;
mod table;
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

use controller::CurrentSprintController;
use state::CurrentSprintState;
use view::CurrentSprintView;

pub struct CurrentSprintScreen {
    state: CurrentSprintState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl CurrentSprintScreen {
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let issues = repo.current_sprint_issues(board_id).await?;
        Ok(Self {
            state: CurrentSprintState::new(issues),
            actions: Arc::new(vec![]),
            mode,
        })
    }
}

impl Screen for CurrentSprintScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        let layout = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(frame.area());
        let content_height = layout[1].height.saturating_sub(3);
        let rows_visible = (content_height / 2).max(1) as usize;
        CurrentSprintController::update_rows_visible(&mut self.state, rows_visible);
        CurrentSprintView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Current Sprint"
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

impl KeyHandler for CurrentSprintScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        CurrentSprintController::handle_command(&mut self.state, command)
    }
}
