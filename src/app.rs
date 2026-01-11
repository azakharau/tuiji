use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::DefaultTerminal;

use crate::{
    app::{
        command_router::CommandRouter,
        input::{CommandLineState, CommandResolver, InputParser, is_question_mark},
        key_handlers::KeyBindings,
        notification_service::NotificationService,
        render::{AppRenderer, RenderState},
        screen_manager::ScreenManager,
        state::{Mode, ScreenType},
        worker_controller::WorkerController,
    },
    config::{AppConfig, AppConfigState},
    data::{AppRepository, RepositoryHub, SqliteRepository, SqliteRepositoryConfig},
    ui::screens::ScreenState,
};

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
pub mod state;
pub mod worker_controller;
pub mod workers;

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

    pub async fn run(&mut self) -> Result<()> {
        event_loop::run(self).await
    }

    async fn handle_input(&mut self, key: KeyEvent) -> Result<ScreenState> {
        if self.notification_service.has_error() {
            self.notification_service.clear_error();
            return Ok(ScreenState::Refresh);
        }
        if self.show_hints && !is_question_mark(&key) {
            self.show_hints = false;
        }
        let had_prefix = self.input.pending_prefix().is_some();
        let bindings = self
            .key_bindings
            .bindings_for_screen(self.state.current_screen);
        // Parse -> resolve -> dispatch: raw key -> parsed input -> command -> handler.
        let event = self.input.parse(key, self.state.mode, bindings.as_slice());
        let has_prefix = self.input.pending_prefix().is_some();

        let Some(event) = event else {
            if had_prefix != has_prefix || has_prefix {
                return Ok(ScreenState::Refresh);
            }
            return Ok(ScreenState::Stay);
        };

        let resolver = CommandResolver::new(&self.key_bindings);
        let Some(cmd) = resolver.resolve(event, self.state.current_screen) else {
            return Ok(ScreenState::Stay);
        };
        let mut router = CommandRouter::new(
            &mut self.state,
            &mut self.screen_stack,
            &mut self.screen_manager,
            &mut self.terminal,
            &mut self.cfg_state,
            &mut self.key_bindings,
            &mut self.repo,
            &mut self.notification_service,
            &mut self.worker_controller,
            &mut self.command_line,
            &mut self.input,
            &mut self.show_hints,
        );
        router.handle_input_command(cmd).await
    }

    async fn render(&mut self) -> Result<()> {
        let render_stack = crate::app::navigation::build_render_stack(
            self.state.current_screen,
            &self.screen_stack,
        );
        let board_required = if crate::app::navigation::board_required_active(&self.state) {
            Some(crate::app::navigation::board_required_bindings(
                self.state.current_screen,
                self.key_bindings.as_ref(),
            ))
        } else {
            None
        };
        let repo = self.repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot render screens")
        })?;
        let render_state = RenderState {
            cfg_state: &self.cfg_state,
            app_state: &self.state,
            repo,
            render_stack,
            key_bindings: self.key_bindings.as_ref(),
            error: self.notification_service.error_state(),
            notifications: self.notification_service.items(),
            command_buffer: self.command_line.buffer(),
            pending_prefix: self.input.pending_prefix(),
            show_hints: self.show_hints,
            board_required,
            mode: self.state.mode,
            sync_paused: self.worker_controller.is_paused(),
            sync_error: self.worker_controller.last_error(),
            sync_status: self.worker_controller.snapshot(),
        };
        AppRenderer::prepare(&mut self.screen_manager, &render_state).await?;
        AppRenderer::draw(&mut self.screen_manager, &render_state, &mut self.terminal)?;
        Ok(())
    }

    fn enforce_command_mode_allowed(&mut self) {
        if self.state.mode == Mode::Command && matches!(self.state.current_screen, ScreenType::Home)
        {
            self.state.mode = Mode::Normal;
            self.input.clear_pending();
            self.command_line.stop();
        }
    }

    async fn init_db(&mut self) -> Result<()> {
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

    fn init_start_screen(&mut self) {
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
