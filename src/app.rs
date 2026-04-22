use std::sync::Arc;

use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;

use crate::{
    app::{
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

mod bootstrap;
mod input_dispatch;

pub mod command_router;
pub mod error;
pub mod event;
pub mod event_loop;
pub mod input;
pub mod key_handlers;
pub mod navigation;
pub mod notification;
pub mod notification_service;
pub mod overlay;
pub mod render;
pub mod screen_manager;
pub(crate) mod screen_policy;
pub mod services;
pub mod state;
pub mod worker_controller;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub mode: state::Mode,
    pub current_screen: state::ScreenType,
    pub selected_board_id: Option<u64>,
    pub profile_editor: Option<ProfileEditorIntent>,
    pub conflict_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileEditorIntent {
    New,
    Edit(String),
}

pub(crate) enum ActionOutcome {
    Continue { render: bool },
    Quit,
}

pub struct App {
    pub terminal: DefaultTerminal,
    pub state: AppState,
    screen_manager: ScreenManager,
    cfg_state: AppConfigState,
    repo: Option<Arc<RepositoryHub>>,
    input: InputParser,
    key_bindings: Arc<KeyBindings>,
    screen_stack: Vec<ScreenType>,
    command_line: CommandLineState,
    show_hints: bool,
    notification_service: NotificationService,
    worker_controller: WorkerController,
}

impl App {
    pub async fn run(&mut self) -> Result<()> {
        event_loop::run(self).await
    }
}
