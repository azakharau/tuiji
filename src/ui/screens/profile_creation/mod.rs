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
    config::ProfileConfig,
    ui::{
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::ProfileCreationController;
use state::ProfileCreationState;
use view::ProfileCreationView;

pub struct ProfileCreationScreen {
    state: ProfileCreationState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl ProfileCreationScreen {
    pub fn new(profile: Option<ProfileConfig>) -> Self {
        Self {
            state: ProfileCreationState::new(profile),
            actions: Arc::new(Vec::new()),
            mode: Mode::Normal,
        }
    }

    pub fn set_profile_id(&mut self, id: String) {
        self.state.set_profile_id(id);
    }
}

impl Screen for ProfileCreationScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        ProfileCreationView::draw(frame, &self.state, self.mode, &self.actions, context);
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
        ProfileCreationController::handle_command_line(&mut self.state, cmd)
    }
}

impl KeyHandler for ProfileCreationScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        ProfileCreationController::handle_command(&mut self.state, command)
    }
}
