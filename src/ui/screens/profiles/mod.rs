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
    ui::{context::RenderContext, screens::{Screen, ScreenState}},
};

use controller::ProfilesController;
use state::ProfilesState;
use view::ProfilesView;

pub struct ProfilesScreen {
    state: ProfilesState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl ProfilesScreen {
    pub fn new(profiles: &[ProfileConfig], active_id: Option<&str>) -> Self {
        Self {
            state: ProfilesState::new(profiles, active_id),
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
        }
    }

    pub fn selected_profile_id(&self) -> Option<&str> {
        self.state.selected_profile_id()
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
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

    pub fn selected_menu_id(&self) -> Option<&'static str> {
        self.state.selected_menu_id()
    }

    pub fn refresh(
        &mut self,
        profiles: &[ProfileConfig],
        active_id: Option<&str>,
        selected_id: Option<&str>,
    ) {
        self.state.refresh(profiles, active_id, selected_id);
    }
}

impl Screen for ProfilesScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        ProfilesView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Profiles"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl KeyHandler for ProfilesScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        ProfilesController::handle_command(&mut self.state, command)
    }
}
