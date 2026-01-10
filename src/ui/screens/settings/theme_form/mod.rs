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
        screens::{CommandLineCommand, Screen, ScreenState},
        theme::{ThemePalette, ThemeRegistry},
    },
};

use controller::SettingsThemeFormController;
use state::SettingsThemeFormState;
use view::SettingsThemeFormView;

pub struct SettingsThemeFormScreen {
    state: SettingsThemeFormState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl SettingsThemeFormScreen {
    pub fn new(active_theme: &str, custom_themes: &[CustomThemeConfig]) -> Self {
        let palette = resolve_palette(active_theme, custom_themes);
        let mut existing_ids = ThemeRegistry::themes()
            .into_iter()
            .map(|theme| theme.id)
            .collect::<Vec<_>>();
        existing_ids.extend(custom_themes.iter().map(|theme| theme.id.clone()));
        Self {
            state: SettingsThemeFormState::new(palette, existing_ids),
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
        }
    }
}

impl Screen for SettingsThemeFormScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        SettingsThemeFormView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Custom Theme"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        SettingsThemeFormController::handle_command_line(&mut self.state, cmd)
    }
}

impl KeyHandler for SettingsThemeFormScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        SettingsThemeFormController::handle_command(&mut self.state, command)
    }
}

fn resolve_palette(active_theme: &str, custom: &[CustomThemeConfig]) -> ThemePalette {
    if let Some(theme) = custom.iter().find(|t| t.id == active_theme) {
        ThemeRegistry::custom_palette(theme).unwrap_or_else(|| ThemeRegistry::get(active_theme))
    } else {
        ThemeRegistry::get(active_theme)
    }
}
