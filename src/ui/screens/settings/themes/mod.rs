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
    config::CustomThemeConfig,
    ui::{
        context::RenderContext,
        screens::{Screen, ScreenState},
    },
};

use controller::SettingsThemesController;
use state::SettingsThemesState;
use view::SettingsThemesView;

pub struct SettingsThemesScreen {
    state: SettingsThemesState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl SettingsThemesScreen {
    pub fn new(active_theme: &str, custom_themes: &[CustomThemeConfig]) -> Self {
        Self {
            state: SettingsThemesState::new(active_theme, custom_themes),
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
        }
    }

    pub fn set_active_theme(&mut self, theme_id: &str) {
        self.state.set_active_theme(theme_id);
    }
}

impl Screen for SettingsThemesScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        SettingsThemesView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Themes"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl KeyHandler for SettingsThemesScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        SettingsThemesController::handle_command(&mut self.state, command)
    }
}
