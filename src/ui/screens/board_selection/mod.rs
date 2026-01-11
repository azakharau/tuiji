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
    data::BoardSummary,
    ui::{
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::BoardSelectionController;
use state::BoardSelectionState;
use view::BoardSelectionView;

pub struct BoardSelectionScreen {
    state: BoardSelectionState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl BoardSelectionScreen {
    pub fn new(boards: Vec<BoardSummary>) -> Self {
        Self {
            state: BoardSelectionState::new(boards),
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
        }
    }

    pub fn selected_board_id(&self) -> Option<u64> {
        self.state.selected_board_id()
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

    pub fn set_items(&mut self, items: Vec<ratatui::widgets::ListItem<'static>>) {
        self.state.set_items(items);
    }

    pub fn refresh_items(&mut self, labels: &[String]) {
        self.state.refresh_items(labels);
    }
}

impl Screen for BoardSelectionScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        BoardSelectionView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Board Selection"
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

impl KeyHandler for BoardSelectionScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        BoardSelectionController::handle_command(&mut self.state, command)
    }
}
