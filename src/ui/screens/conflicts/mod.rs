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

use controller::ConflictsController;
use state::ConflictsState;
use view::ConflictsView;

pub struct ConflictsScreen {
    state: ConflictsState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl ConflictsScreen {
    pub fn new(issues: Vec<IssueSummary>) -> Self {
        Self {
            state: ConflictsState::new(issues),
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
        }
    }

    pub fn move_up(&mut self, n: usize) {
        self.state.move_up(n);
    }

    pub fn move_down(&mut self, n: usize) {
        self.state.move_down(n);
    }

    pub fn move_top(&mut self) {
        self.state.move_top();
    }

    pub fn move_bottom(&mut self) {
        self.state.move_bottom();
    }

    pub fn selected_issue_key(&self) -> Option<&str> {
        self.state.selected_issue_key()
    }

    pub fn set_issues(&mut self, issues: Vec<IssueSummary>) {
        self.state.set_issues(issues);
    }
}

impl Screen for ConflictsScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        ConflictsView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Conflicts Screen"
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

impl KeyHandler for ConflictsScreen {
    fn handle_command(&mut self, command: Command) -> crate::ui::screens::ScreenState {
        ConflictsController::handle_command(&mut self.state, command)
    }
}
