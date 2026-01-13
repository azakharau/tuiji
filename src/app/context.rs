use std::sync::Arc;

use ratatui::DefaultTerminal;

use crate::{
    app::{
        AppState,
        input::{CommandLineState, InputParser},
        key_handlers::KeyBindings,
        notification_service::NotificationService,
        screen_manager::ScreenManager,
        state::ScreenType,
        worker_controller::WorkerController,
    },
    config::AppConfigState,
    data::RepositoryHub,
};

/// Core application context containing state and screen management
pub struct AppContext<'a> {
    pub state: &'a mut AppState,
    pub screen_stack: &'a mut Vec<ScreenType>,
    pub screen_manager: &'a mut ScreenManager,
    pub terminal: &'a mut DefaultTerminal,
    pub cfg_state: &'a mut AppConfigState,
    pub key_bindings: &'a mut Arc<KeyBindings>,
    pub repo: &'a mut Option<Arc<RepositoryHub>>,
}

/// Service context containing notification and worker services
pub struct ServiceContext<'a> {
    pub notification: &'a mut NotificationService,
    pub worker: &'a mut WorkerController,
}

/// Input context containing command line and input parsing state
pub struct InputContext<'a> {
    pub command_line: &'a mut CommandLineState,
    pub input: &'a mut InputParser,
    pub show_hints: &'a mut bool,
}
