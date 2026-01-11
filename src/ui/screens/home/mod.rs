mod controller;
mod state;
mod view;

use ratatui::Frame;

use crate::{
    app::key_handlers::{ActionHint, Command, KeyHandler},
    config::AppConfigState,
    ui::{
        components::logo::AsciiLogoComponent,
        context::RenderContext,
        screens::{Screen, ScreenState},
    },
};

use controller::HomeController;
use state::HomeState;
use view::HomeView;

pub struct HomeScreen {
    state: HomeState,
    logo: AsciiLogoComponent,
}

impl HomeScreen {
    pub fn new(logo: AsciiLogoComponent, cfg: &AppConfigState, conflict_count: usize) -> Self {
        Self {
            state: HomeState::new(cfg, conflict_count),
            logo,
        }
    }
}

impl Screen for HomeScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        HomeView::draw(frame, &self.state, &self.logo, context);
    }

    fn name(&self) -> &'static str {
        "Home Screen"
    }

    fn set_action_hints(&mut self, _actions: std::sync::Arc<Vec<ActionHint>>) {}
}

impl KeyHandler for HomeScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        HomeController::handle_command(&mut self.state, command)
    }
}
