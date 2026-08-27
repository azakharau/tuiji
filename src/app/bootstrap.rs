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
        let selected = repo.default_board_id().await?;
        self.state.selected_board_id = selected;
        self.state.conflict_count = repo.conflict_count().await.unwrap_or(0);
        self.repo = Some(Arc::new(repo));
        Ok(())
    }

    pub(super) fn init_start_screen(&mut self) {
        let has_usable_profile = match &self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.active_profile().is_some_and(|profile| {
                !profile.jira.base_url.trim().is_empty() && !profile.jira.username.trim().is_empty()
            }),
            AppConfigState::Missing(_) => false,
        };

        self.state.current_screen = if !has_usable_profile {
            ScreenType::ProfileCreation
        } else if self.state.selected_board_id.is_none() {
            ScreenType::BoardSelection
        } else {
            ScreenType::CurrentSprint
        };
    }
}
