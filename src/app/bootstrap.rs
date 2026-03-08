use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;

use super::*;
use crate::{
    app::{
        input::{CommandLineState, InputParser},
        key_handlers::KeyBindings,
        notification_service::NotificationService,
        screen_manager::ScreenManager,
    },
    config::{AppConfig, AppConfigState},
    data::{
        BoardRepository, ConflictRepository, RepositoryHub, SqliteRepository,
        SqliteRepositoryConfig,
    },
};

impl App {
    pub fn new(terminal: DefaultTerminal, state: AppState) -> Result<Self> {
        let config = AppConfig::load_state();
        let key_bindings = match &config {
            AppConfigState::Loaded(cfg) => Arc::new(KeyBindings::from_config(&cfg.keybindings)),
            AppConfigState::Missing(_) => {
                Arc::new(KeyBindings::from_config(&AppConfig::default().keybindings))
            }
        };
        let screen_manager = match &config {
            AppConfigState::Loaded(cfg) => {
                ScreenManager::new(Duration::from_secs(cfg.ui.screen_cache_ttl_seconds))
            }
            AppConfigState::Missing(_) => ScreenManager::default(),
        };
        let ui_cfg = match &config {
            AppConfigState::Loaded(cfg) => cfg.ui.clone(),
            AppConfigState::Missing(_) => AppConfig::default().ui,
        };
        let notification_service = NotificationService::from_ui(&ui_cfg);
        Ok(Self {
            terminal,
            state,
            screen_manager,
            cfg_state: config,
            repo: None,
            input: InputParser::default(),
            key_bindings,
            screen_stack: Vec::new(),
            command_line: CommandLineState::new(),
            show_hints: false,
            notification_service,
            worker_controller: WorkerController::new(),
        })
    }

    pub(super) async fn init_db(&mut self) -> Result<()> {
        let cfg = SqliteRepositoryConfig {
            db_path: SqliteRepository::default_db_path(),
        };
        let profile = match &self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.active_profile(),
            AppConfigState::Missing(_) => None,
        };
        let repo = RepositoryHub::connect(cfg, profile).await?;
        let mut selected = repo.default_board_id().await?;
        if selected.is_none() {
            selected = repo.seed_mock_data_if_empty().await?;
        }
        self.state.selected_board_id = selected;
        self.state.conflict_count = repo.conflict_count().await.unwrap_or(0);
        self.repo = Some(Arc::new(repo));
        Ok(())
    }

    pub(super) fn init_start_screen(&mut self) {
        match self.cfg_state {
            AppConfigState::Loaded(_) => {
                self.state.current_screen = ScreenType::Home;
            }
            AppConfigState::Missing(_) => {
                // Placeholder: could route to Welcome screen later.
                self.state.current_screen = ScreenType::Home;
            }
        }
    }
}
