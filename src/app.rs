use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyEvent};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    app::{
        event::{AppEvent, InputEvent, SystemEvent, WorkerEvent},
        input::overlay::{command_line_area, modal_area},
        input::{
            CommandLineAction, CommandLineOutcome, CommandLineState, CommandResolver, InputCommand,
            InputParser, TextInput, is_question_mark,
        },
        key_handlers::{
            ActionId, Command, KeyBindings, action_hints, binding_hints_for_prefix,
            binding_hints_for_screen,
        },
        screen_manager::{ScreenContext, ScreenManager},
        state::{Mode, ScreenType},
    },
    config::{AppConfig, AppConfigState, ProfileConfig},
    data::{AppRepository, RepositoryHub, SqliteRepository, SqliteRepositoryConfig},
    ui::{
        components::{command_line::CommandLine, which_key_popup::WhichKeyPopup},
        screens::{CommandLineCommand, ScreenState},
    },
};

pub mod event;
pub mod input;
pub mod key_handlers;
pub mod screen_manager;
pub mod state;
pub mod workers;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub mode: state::Mode,
    pub current_screen: state::ScreenType,
    pub selected_board_id: Option<u64>,
    pub profile_editor: Option<ProfileEditorIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileEditorIntent {
    New,
    Edit(String),
}

enum ActionOutcome {
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
    error_message: Option<String>,
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
            error_message: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init_db().await?;
        self.terminal.clear()?;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let _input = spawn_input_listener(tx.clone());
        // let _tick = spawn_tick(tx.clone(), Duration::from_millis(250));
        // let _cache_worker = spawn_worker(tx.clone());
        // let _notification_worker = spawn_notifications(tx.clone());

        // Initial paint.
        self.init_start_screen();
        self.render().await?;

        while let Some(event) = rx.recv().await {
            let mut render_requested = false;

            match event {
                AppEvent::Input(InputEvent::Key(key)) => {
                    let action = self.handle_input(key).await?;
                    match self.apply_action(action)? {
                        ActionOutcome::Continue { render } => render_requested |= render,
                        ActionOutcome::Quit => {
                            break;
                        }
                    }
                }
                AppEvent::System(SystemEvent::Tick) => {
                    render_requested = true;
                }
                AppEvent::Worker(msg) => {
                    render_requested |= self.handle_worker(msg)?;
                }
                AppEvent::Ui(_)
                | AppEvent::Nav(_)
                | AppEvent::Repo(_)
                | AppEvent::Notification(_) => {}
            }

            if render_requested {
                self.render().await?;
            }
        }

        Ok(())
    }

    async fn handle_input(&mut self, key: KeyEvent) -> Result<ScreenState> {
        if self.error_message.is_some() {
            self.error_message = None;
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
        self.handle_input_command(cmd).await
    }

    async fn handle_input_command(&mut self, event: InputCommand) -> Result<ScreenState> {
        match event {
            InputCommand::Action(cmd) => self.handle_action_command(cmd).await,
            InputCommand::ModeSwitch(mode) => {
                if mode == Mode::Normal
                    && self.state.mode == Mode::Normal
                    && self.is_modal_screen(self.state.current_screen)
                {
                    return Ok(ScreenState::Close);
                }
                if mode == Mode::Command && !self.command_mode_allowed() {
                    return Ok(ScreenState::Stay);
                }
                self.set_mode(mode);
                Ok(ScreenState::Refresh)
            }
            InputCommand::ToggleHints => {
                self.show_hints = !self.show_hints;
                Ok(ScreenState::Refresh)
            }
            InputCommand::Text(text) => match self.state.mode {
                Mode::Command => self.handle_command_line_event(text).await,
                Mode::Insert => self.handle_insert_input(text).await,
                _ => Ok(ScreenState::Stay),
            },
        }
    }

    async fn handle_action_command(&mut self, cmd: Command) -> Result<ScreenState> {
        if let ActionId::EnterInsert(_) = cmd.action {
            self.set_mode(Mode::Insert);
        }
        if self.state.selected_board_id.is_none()
            && !matches!(self.state.current_screen, ScreenType::BoardSelection)
        {
            return self.handle_board_required_action(cmd).await;
        }
        if let Some(nav) = self.map_global_action(&cmd) {
            return Ok(nav);
        }
        if self.state.current_screen == ScreenType::BoardSelection {
            return self.handle_board_selection_command(cmd).await;
        }
        if self.state.current_screen == ScreenType::Profiles {
            return self.handle_profiles_command(cmd).await;
        }

        let repo = self.repo.clone().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot dispatch screen command")
        })?;
        let ctx = ScreenContext {
            cfg_state: &self.cfg_state,
            app_state: &self.state,
            repo,
        };
        let screen = self
            .screen_manager
            .active_screen_mut(self.state.current_screen, ctx)
            .await?;
        Ok(screen.handle_command(cmd))
    }

    async fn handle_insert_input(&mut self, input: TextInput) -> Result<ScreenState> {
        if matches!(input, TextInput::Esc) {
            self.set_mode(Mode::Normal);
            return Ok(ScreenState::Refresh);
        }
        let key = match input {
            TextInput::Char(ch) => crossterm::event::KeyCode::Char(ch),
            TextInput::Backspace => crossterm::event::KeyCode::Backspace,
            TextInput::Delete => crossterm::event::KeyCode::Delete,
            TextInput::Enter => crossterm::event::KeyCode::Enter,
            TextInput::Tab => crossterm::event::KeyCode::Tab,
            TextInput::Esc => crossterm::event::KeyCode::Esc,
        };
        self.forward_raw_input(key).await
    }

    async fn handle_board_required_action(&mut self, cmd: Command) -> Result<ScreenState> {
        match cmd.action {
            ActionId::OpenBoards => Ok(ScreenState::SwitchTo(ScreenType::BoardSelection)),
            ActionId::Quit => {
                if self.state.current_screen == ScreenType::Home {
                    Ok(ScreenState::Quit)
                } else {
                    Ok(ScreenState::Stay)
                }
            }
            _ => Ok(ScreenState::Stay),
        }
    }

    async fn forward_raw_input(&mut self, code: crossterm::event::KeyCode) -> Result<ScreenState> {
        let repo = self.repo.clone().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot dispatch raw input")
        })?;
        let ctx = ScreenContext {
            cfg_state: &self.cfg_state,
            app_state: &self.state,
            repo,
        };
        let screen = self
            .screen_manager
            .active_screen_mut(self.state.current_screen, ctx)
            .await?;
        Ok(screen.handle_command(Command {
            action: ActionId::RawInput(code),
            repeat: 1,
        }))
    }

    fn apply_action(&mut self, action: ScreenState) -> Result<ActionOutcome> {
        match action {
            ScreenState::Quit => {
                self.terminal.clear()?;
                Ok(ActionOutcome::Quit)
            }
            ScreenState::SwitchTo(new_screen) => {
                if self.state.current_screen == ScreenType::ProfileCreation
                    && new_screen != ScreenType::ProfileCreation
                {
                    self.screen_manager.invalidate(ScreenType::ProfileCreation);
                    self.state.profile_editor = None;
                }
                if new_screen == ScreenType::Home {
                    self.screen_stack.clear();
                } else if new_screen != self.state.current_screen {
                    self.screen_stack.push(self.state.current_screen);
                }
                self.state.current_screen = new_screen;
                if new_screen == ScreenType::ProfileCreation && self.state.profile_editor.is_none()
                {
                    self.state.profile_editor = Some(ProfileEditorIntent::New);
                }
                if self.state.mode == Mode::Command && !self.command_mode_allowed() {
                    self.set_mode(Mode::Normal);
                }
                self.terminal.clear()?;
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::Refresh => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::Stay => Ok(ActionOutcome::Continue { render: false }),
            ScreenState::SaveProfile(profile) => {
                let profile_id = profile.id.clone();
                if let Err(err) = self.save_profile(profile) {
                    self.error_message = Some(err.to_string());
                    return Ok(ActionOutcome::Continue { render: true });
                }
                self.state.profile_editor = Some(ProfileEditorIntent::Edit(profile_id.clone()));
                if let Some(screen) = self.screen_manager.profile_creation_mut() {
                    screen.set_profile_id(profile_id);
                }
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::SaveProfileAndClose(profile) => {
                if let Err(err) = self.save_profile(profile) {
                    self.error_message = Some(err.to_string());
                    return Ok(ActionOutcome::Continue { render: true });
                }
                self.close_screen()
            }
            ScreenState::Close => self.close_screen(),
        }
    }

    fn handle_worker(&mut self, _msg: WorkerEvent) -> Result<bool> {
        // Placeholder: update state based on worker notifications when added.
        Ok(true)
    }

    fn map_global_action(&mut self, cmd: &Command) -> Option<ScreenState> {
        match cmd.action {
            ActionId::Quit => {
                if self.state.current_screen == ScreenType::Home {
                    Some(ScreenState::Quit)
                } else {
                    None
                }
            }
            ActionId::Refresh => Some(ScreenState::Refresh),
            ActionId::GoHome => Some(ScreenState::SwitchTo(ScreenType::Home)),
            ActionId::OpenCurrentSprint => Some(ScreenState::SwitchTo(ScreenType::CurrentSprint)),
            ActionId::OpenProfiles => Some(ScreenState::SwitchTo(ScreenType::Profiles)),
            ActionId::OpenMyIssues => Some(ScreenState::SwitchTo(ScreenType::MyIssues)),
            ActionId::OpenSearchIssues => Some(ScreenState::SwitchTo(ScreenType::SearchIssues)),
            ActionId::OpenNewIssue => Some(ScreenState::SwitchTo(ScreenType::NewIssue)),
            ActionId::OpenBoards => Some(ScreenState::SwitchTo(ScreenType::BoardSelection)),
            _ => None,
        }
    }

    async fn render(&mut self) -> Result<()> {
        let screen_type = self.state.current_screen;
        let render_stack = self.build_render_stack();
        let show_board_required = self.state.selected_board_id.is_none()
            && !matches!(self.state.current_screen, ScreenType::BoardSelection);
        let error_message = self.error_message.clone();
        let show_hints = self.show_hints;
        let pending_prefix = self.input.pending_prefix();
        let command_buffer = self.command_line.buffer().map(|buf| buf.to_string());
        let board_required_bindings = if show_board_required {
            Some(self.board_required_bindings(screen_type))
        } else {
            None
        };
        let hint_popup = if show_hints {
            Some(WhichKeyPopup::new(
                "Key Hints".to_string(),
                binding_hints_for_screen(screen_type, &self.key_bindings),
            ))
        } else if let Some(prefix) = pending_prefix.as_deref() {
            Some(WhichKeyPopup::new(
                format!("Keys: {prefix}"),
                binding_hints_for_prefix(screen_type, prefix, &self.key_bindings),
            ))
        } else {
            None
        };
        let repo = self.repo.clone().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot render screens")
        })?;
        for screen_type in &render_stack {
            let ctx = ScreenContext {
                cfg_state: &self.cfg_state,
                app_state: &self.state,
                repo: repo.clone(),
            };
            let _ = self
                .screen_manager
                .active_screen_mut(*screen_type, ctx)
                .await?;
        }
        let key_bindings = self.key_bindings.clone();
        let mode = self.state.mode;
        let screen_manager = &mut self.screen_manager;
        let cmd_color = Mode::Command.color();
        self.terminal.draw(|frame| {
            for screen_type in &render_stack {
                if let Some(screen) = screen_manager.screen_mut_existing(*screen_type) {
                    let hints = action_hints(*screen_type, &key_bindings);
                    screen.set_action_hints(hints);
                    screen.set_mode(mode);
                    screen.draw(frame);
                }
            }
            if let Some(msg) = &error_message {
                let height = 5.min(frame.area().height);
                let area = modal_area(frame.area(), 60.min(frame.area().width), height);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title(Line::from("Error").centered())
                    .title_style(Style::default().fg(Color::Red));
                frame.render_widget(Clear, area);
                frame.render_widget(&block, area);
                let inner = block.inner(area);
                let text = Paragraph::new(msg.as_str())
                    .alignment(Alignment::Center)
                    .wrap(Wrap::default());
                frame.render_widget(text, inner);
            } else if show_board_required {
                let height = 7.min(frame.area().height);
                let area = modal_area(frame.area(), 60.min(frame.area().width), height);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(cmd_color))
                    .title(Line::from("Board Required").centered())
                    .title_style(Style::default().fg(cmd_color));
                frame.render_widget(Clear, area);
                frame.render_widget(&block, area);
                let inner = block.inner(area);
                let sections =
                    Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(inner);
                let text = Paragraph::new("No board selected.\nConfigure a board to continue.")
                    .alignment(Alignment::Center)
                    .wrap(Wrap::default());
                frame.render_widget(text, sections[0]);
                let (open_key, quit_key) = board_required_bindings
                    .clone()
                    .unwrap_or_else(|| ("b".to_string(), "q".to_string()));
                let options =
                    Paragraph::new(format!("[{open_key}] Configure boards\n[{quit_key}] Quit"))
                        .alignment(Alignment::Center)
                        .wrap(Wrap::default());
                frame.render_widget(options, sections[1]);
            } else if let Some(popup) = &hint_popup {
                frame.render_widget(popup, frame.area());
            }
            if let Some(buffer) = &command_buffer {
                let area = command_line_area(frame.area());
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(cmd_color))
                    .title(Line::from("Command").centered())
                    .title_style(Style::default().fg(cmd_color));
                frame.render_widget(Clear, area);
                frame.render_widget(&block, area);
                frame.render_widget(CommandLine::new(buffer, cmd_color), block.inner(area));
            }
        })?;
        Ok(())
    }

    fn build_render_stack(&self) -> Vec<ScreenType> {
        match self.state.current_screen {
            ScreenType::Profiles | ScreenType::ProfileCreation | ScreenType::BoardSelection => {
                let mut stack = self.screen_stack.clone();
                stack.push(self.state.current_screen);
                stack
            }
            _ => vec![self.state.current_screen],
        }
    }

    fn is_modal_screen(&self, screen: ScreenType) -> bool {
        matches!(
            screen,
            ScreenType::Profiles | ScreenType::ProfileCreation | ScreenType::BoardSelection
        )
    }

    fn has_modal_stack(&self) -> bool {
        self.is_modal_screen(self.state.current_screen)
            || self
                .screen_stack
                .iter()
                .any(|screen| self.is_modal_screen(*screen))
    }

    fn close_all_modals(&mut self) -> Result<ScreenState> {
        if !self.has_modal_stack() {
            return Ok(ScreenState::Stay);
        }
        let target = self
            .screen_stack
            .iter()
            .rev()
            .find(|screen| !self.is_modal_screen(**screen))
            .copied()
            .unwrap_or(ScreenType::Home);
        self.screen_stack.clear();
        if self.is_modal_screen(self.state.current_screen) {
            self.state.profile_editor = None;
        }
        self.state.current_screen = target;
        if self.state.mode == Mode::Command && !self.command_mode_allowed() {
            self.set_mode(Mode::Normal);
        }
        self.terminal.clear()?;
        Ok(ScreenState::Refresh)
    }

    fn board_required_bindings(&self, screen: ScreenType) -> (String, String) {
        let bindings = self.key_bindings.bindings_for_screen(screen);
        let open_key = bindings
            .iter()
            .find(|entry| entry.action == ActionId::OpenBoards)
            .map(|entry| entry.binding.clone())
            .unwrap_or_else(|| "b".to_string());
        let quit_key = bindings
            .iter()
            .find(|entry| entry.action == ActionId::Quit)
            .map(|entry| entry.binding.clone())
            .unwrap_or_else(|| "q".to_string());
        (open_key, quit_key)
    }

    fn set_mode(&mut self, mode: Mode) {
        self.state.mode = mode;
        self.input.clear_pending();
        if mode == Mode::Command {
            if !self.command_mode_allowed() {
                self.state.mode = Mode::Normal;
                self.command_line.stop();
                return;
            }
            self.command_line.start();
        } else {
            self.command_line.stop();
        }
    }

    async fn handle_command_line_event(&mut self, event: TextInput) -> Result<ScreenState> {
        match self.command_line.handle_event(event) {
            CommandLineOutcome::Updated => Ok(ScreenState::Refresh),
            CommandLineOutcome::Cancelled => {
                self.set_mode(Mode::Normal);
                Ok(ScreenState::Refresh)
            }
            CommandLineOutcome::Submitted(action) => {
                self.set_mode(Mode::Normal);
                if let Some(action) = action {
                    self.handle_command_line_action(action).await
                } else {
                    Ok(ScreenState::Stay)
                }
            }
            CommandLineOutcome::Noop => Ok(ScreenState::Stay),
        }
    }

    async fn handle_command_line_action(
        &mut self,
        action: CommandLineAction,
    ) -> Result<ScreenState> {
        match action {
            CommandLineAction::Write => {
                let repo = self.repo.clone().ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Repository not initialized: cannot handle command line action"
                    )
                })?;
                let ctx = ScreenContext {
                    cfg_state: &self.cfg_state,
                    app_state: &self.state,
                    repo,
                };
                let screen = self
                    .screen_manager
                    .active_screen_mut(self.state.current_screen, ctx)
                    .await?;
                Ok(screen.handle_command_line(CommandLineCommand::Write))
            }
            CommandLineAction::WriteQuit => {
                if !self.has_modal_stack() {
                    let repo = self.repo.clone().ok_or_else(|| {
                        color_eyre::eyre::eyre!(
                            "Repository not initialized: cannot handle command line action"
                        )
                    })?;
                    let ctx = ScreenContext {
                        cfg_state: &self.cfg_state,
                        app_state: &self.state,
                        repo,
                    };
                    let screen = self
                        .screen_manager
                        .active_screen_mut(self.state.current_screen, ctx)
                        .await?;
                    let res = screen.handle_command_line(CommandLineCommand::Write);
                    if let ScreenState::SaveProfile(profile)
                    | ScreenState::SaveProfileAndClose(profile) = res
                    {
                        self.save_profile(profile)?;
                        return Ok(ScreenState::Refresh);
                    }
                    return Ok(ScreenState::Stay);
                }
                let repo = self.repo.clone().ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Repository not initialized: cannot handle command line action"
                    )
                })?;
                let ctx = ScreenContext {
                    cfg_state: &self.cfg_state,
                    app_state: &self.state,
                    repo,
                };
                let screen = self
                    .screen_manager
                    .active_screen_mut(self.state.current_screen, ctx)
                    .await?;
                Ok(screen.handle_command_line(CommandLineCommand::WriteQuit))
            }
            CommandLineAction::WriteQuitAll => {
                if !self.has_modal_stack() {
                    return Ok(ScreenState::Stay);
                }
                let repo = self.repo.clone().ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Repository not initialized: cannot handle command line action"
                    )
                })?;
                let ctx = ScreenContext {
                    cfg_state: &self.cfg_state,
                    app_state: &self.state,
                    repo,
                };
                let screen = self
                    .screen_manager
                    .active_screen_mut(self.state.current_screen, ctx)
                    .await?;
                let res = screen.handle_command_line(CommandLineCommand::Write);
                if let ScreenState::SaveProfile(profile)
                | ScreenState::SaveProfileAndClose(profile) = res
                {
                    self.save_profile(profile)?;
                }
                self.close_all_modals()
            }
            CommandLineAction::Quit => {
                if !self.has_modal_stack() {
                    return Ok(ScreenState::Stay);
                }
                if self.screen_stack.is_empty() && self.is_modal_screen(self.state.current_screen) {
                    return self.close_all_modals();
                }
                Ok(ScreenState::Close)
            }
            CommandLineAction::QuitAll => self.close_all_modals(),
        }
    }

    fn save_config(&mut self, cfg: AppConfig) -> Result<()> {
        cfg.save()?;
        self.key_bindings = Arc::new(KeyBindings::from_config(&cfg.keybindings));
        if let Some(repo) = &self.repo {
            self.repo = Some(Arc::new(repo.with_profile(cfg.active_profile())?));
        } else {
            return Err(color_eyre::eyre::eyre!(
                "Repository not initialized: cannot refresh active profile"
            ));
        }
        self.cfg_state = AppConfigState::Loaded(cfg);
        self.screen_manager.invalidate_many(&[
            ScreenType::Home,
            ScreenType::CurrentSprint,
            ScreenType::Profiles,
        ]);
        Ok(())
    }

    fn save_profile(&mut self, profile: ProfileConfig) -> Result<()> {
        let mut cfg = match &self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.clone(),
            AppConfigState::Missing(_) => AppConfig::default(),
        };
        let profile_id = profile.id.clone();
        cfg.upsert_profile(profile);
        cfg.set_active_profile(&profile_id);
        self.save_config(cfg)?;
        Ok(())
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
        self.repo = Some(Arc::new(repo));
        Ok(())
    }

    async fn handle_board_selection_command(&mut self, cmd: Command) -> Result<ScreenState> {
        let screen = self.ensure_board_selection_screen().await?;

        match cmd.action {
            ActionId::MoveUp => {
                screen.move_up(cmd.repeat);
                Ok(ScreenState::Refresh)
            }
            ActionId::MoveDown => {
                screen.move_down(cmd.repeat);
                Ok(ScreenState::Refresh)
            }
            ActionId::MoveTop => {
                screen.move_top();
                Ok(ScreenState::Refresh)
            }
            ActionId::MoveBottom => {
                screen.move_bottom();
                Ok(ScreenState::Refresh)
            }
            ActionId::Confirm => {
                let Some(board_id) = screen.selected_board_id() else {
                    return Ok(ScreenState::Stay);
                };
                let repo = self.repo.clone().ok_or_else(|| {
                    color_eyre::eyre::eyre!("Repository not initialized: cannot select board")
                })?;
                repo.set_selected_board(board_id, true).await?;
                self.state.selected_board_id = Some(board_id);
                self.screen_manager
                    .invalidate_many(&[ScreenType::CurrentSprint, ScreenType::BoardSelection]);
                self.screen_stack.clear();
                Ok(ScreenState::SwitchTo(ScreenType::Home))
            }
            _ => Ok(ScreenState::Stay),
        }
    }

    async fn handle_profiles_command(&mut self, cmd: Command) -> Result<ScreenState> {
        match cmd.action {
            ActionId::MoveUp => {
                let screen = self.screen_manager.ensure_profiles(&self.cfg_state);
                screen.move_up(cmd.repeat);
                return Ok(ScreenState::Refresh);
            }
            ActionId::MoveDown => {
                let screen = self.screen_manager.ensure_profiles(&self.cfg_state);
                screen.move_down(cmd.repeat);
                return Ok(ScreenState::Refresh);
            }
            ActionId::MoveTop => {
                let screen = self.screen_manager.ensure_profiles(&self.cfg_state);
                screen.move_top();
                return Ok(ScreenState::Refresh);
            }
            ActionId::MoveBottom => {
                let screen = self.screen_manager.ensure_profiles(&self.cfg_state);
                screen.move_bottom();
                return Ok(ScreenState::Refresh);
            }
            _ => {}
        }

        let (is_empty, selected_menu, selected_profile) = {
            let screen = self.screen_manager.ensure_profiles(&self.cfg_state);
            (
                screen.is_empty(),
                screen.selected_menu_id(),
                screen.selected_profile_id().map(|id| id.to_string()),
            )
        };

        match cmd.action {
            ActionId::Confirm => {
                if is_empty {
                    if matches!(selected_menu, Some("quit")) {
                        return Ok(ScreenState::Quit);
                    }
                    self.start_profile_creation(ProfileEditorIntent::New);
                    return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
                }
                let Some(profile_id) = selected_profile else {
                    return Ok(ScreenState::Stay);
                };
                let mut cfg = match &self.cfg_state {
                    AppConfigState::Loaded(cfg) => cfg.clone(),
                    AppConfigState::Missing(_) => AppConfig::default(),
                };
                if cfg.profiles.iter().any(|p| p.id == profile_id) {
                    cfg.set_active_profile(&profile_id);
                    self.save_config(cfg)?;
                }
                Ok(ScreenState::Refresh)
            }
            ActionId::EditProfile => {
                if is_empty {
                    self.start_profile_creation(ProfileEditorIntent::New);
                    return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
                }
                if let (AppConfigState::Loaded(cfg), Some(id)) = (&self.cfg_state, selected_profile)
                {
                    if cfg.profiles.iter().any(|p| p.id == id) {
                        self.start_profile_creation(ProfileEditorIntent::Edit(id));
                        return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
                    }
                }
                Ok(ScreenState::Stay)
            }
            ActionId::DeleteProfile => {
                if is_empty {
                    return Ok(ScreenState::Stay);
                }
                let Some(profile_id) = selected_profile else {
                    return Ok(ScreenState::Stay);
                };
                let mut cfg = match &self.cfg_state {
                    AppConfigState::Loaded(cfg) => cfg.clone(),
                    AppConfigState::Missing(_) => AppConfig::default(),
                };
                if cfg.remove_profile(&profile_id) {
                    self.save_config(cfg)?;
                    return Ok(ScreenState::Refresh);
                }
                Ok(ScreenState::Stay)
            }
            ActionId::NewProfile => {
                self.start_profile_creation(ProfileEditorIntent::New);
                Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation))
            }
            _ => Ok(ScreenState::Stay),
        }
    }

    fn start_profile_creation(&mut self, intent: ProfileEditorIntent) {
        self.state.profile_editor = Some(intent);
        self.screen_manager.invalidate(ScreenType::ProfileCreation);
    }

    async fn ensure_board_selection_screen(
        &mut self,
    ) -> Result<&mut crate::ui::screens::board_selection::BoardSelectionScreen> {
        if self.screen_manager.board_selection_mut().is_none() {
            let repo = self.repo.clone().ok_or_else(|| {
                color_eyre::eyre::eyre!("Repository not initialized: cannot open boards")
            })?;
            let ctx = ScreenContext {
                cfg_state: &self.cfg_state,
                app_state: &self.state,
                repo,
            };
            let _ = self
                .screen_manager
                .active_screen_mut(ScreenType::BoardSelection, ctx)
                .await?;
        }
        self.screen_manager
            .board_selection_mut()
            .ok_or_else(|| color_eyre::eyre::eyre!("Board selection screen missing"))
    }

    fn command_mode_allowed(&self) -> bool {
        !matches!(self.state.current_screen, ScreenType::Home)
    }

    fn close_screen(&mut self) -> Result<ActionOutcome> {
        if self.state.current_screen == ScreenType::ProfileCreation {
            self.screen_manager.invalidate(ScreenType::ProfileCreation);
            self.state.profile_editor = None;
        }
        if let Some(prev) = self.screen_stack.pop() {
            self.state.current_screen = prev;
            self.terminal.clear()?;
            Ok(ActionOutcome::Continue { render: true })
        } else if self.is_modal_screen(self.state.current_screen) {
            self.state.current_screen = ScreenType::Home;
            self.terminal.clear()?;
            Ok(ActionOutcome::Continue { render: true })
        } else {
            self.terminal.clear()?;
            Ok(ActionOutcome::Quit)
        }
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

fn spawn_input_listener(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(event) = reader.next().await {
            match event {
                Ok(Event::Key(key)) => {
                    let _ = tx.send(AppEvent::Input(InputEvent::Key(key)));
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("event stream error: {err}");
                    break;
                }
            }
        }
    })
}

// fn spawn_tick(tx: UnboundedSender<AppEvent>, period: Duration) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         let mut ticker = time::interval(period);
//         loop {
//             ticker.tick().await;
//             if tx.send(AppEvent::System(SystemEvent::Tick)).is_err() {
//                 break;
//             }
//         }
//     })
// }
//
// fn spawn_worker(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         // Placeholder for cache/notification workers; send messages when ready.
//         // let _ = tx.send(AppEvent::Worker(WorkerEvent::JiraUpdated));
//         let _ = tx;
//     })
// }
//
// fn spawn_notifications(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         let mut ticker = time::interval(NOTIFY_POLL_INTERVAL);
//         loop {
//             ticker.tick().await;
//             // placeholder: poll notification source, then:
//             // if tx.send(AppEvent::Worker(WorkerEvent::Notification(payload))).is_err() { break; }
//             if tx.is_closed() {
//                 break;
//             }
//         }
//     })
// }
