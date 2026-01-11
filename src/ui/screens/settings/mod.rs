mod controller;
mod state;
pub mod theme_form;
pub mod themes;
mod view;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    app::{
        key_handlers::{ActionHint, Command, KeyHandler},
        state::Mode,
    },
    ui::{
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::SettingsController;
use state::SettingsState;
use view::SettingsView;

pub struct SettingsScreen {
    state: SettingsState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            state: SettingsState::new(),
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
}

impl Screen for SettingsScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        SettingsView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Settings"
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

impl KeyHandler for SettingsScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        SettingsController::handle_command(&mut self.state, command)
    }
}
