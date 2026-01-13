use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;

use crate::{
    app::{
        AppState, ProfileEditorIntent,
        error::{AppErrorLevel, AppErrorState},
        input::{
            CommandLineAction, CommandLineOutcome, CommandLineState, InputCommand, InputParser,
            SyncAction, TextInput,
        },
        key_handlers::{ActionId, Command, KeyBindings, KeyHandler},
        navigation::{
            board_required_active, close_all_modals_impl, has_modal_stack, is_modal_screen,
        },
        notification::AppNotificationKind,
        notification_service::NotificationService,
        screen_manager::{ScreenContext, ScreenManager},
        state::{Mode, ScreenType},
        worker_controller::{SyncJob, SyncJobKind, SyncSource, WorkerController},
    },
    config::{AppConfig, AppConfigState, ProfileConfig, SyncMode},
    data::{AppRepository, RepositoryHub, repository::CommandRepository},
    ui::screens::{CommandLineCommand, ScreenState},
};

pub struct CommandRouter<'a> {
    state: &'a mut AppState,
    screen_stack: &'a mut Vec<ScreenType>,
    screen_manager: &'a mut ScreenManager,
    terminal: &'a mut DefaultTerminal,
    cfg_state: &'a mut AppConfigState,
    key_bindings: &'a mut Arc<KeyBindings>,
    repo: &'a mut Option<Arc<RepositoryHub>>,
    notification_service: &'a mut NotificationService,
    worker_controller: &'a mut WorkerController,
    command_line: &'a mut CommandLineState,
    input: &'a mut InputParser,
    show_hints: &'a mut bool,
}

impl<'a> CommandRouter<'a> {
    pub fn new(
        state: &'a mut AppState,
        screen_stack: &'a mut Vec<ScreenType>,
        screen_manager: &'a mut ScreenManager,
        terminal: &'a mut DefaultTerminal,
        cfg_state: &'a mut AppConfigState,
        key_bindings: &'a mut Arc<KeyBindings>,
        repo: &'a mut Option<Arc<RepositoryHub>>,
        notification_service: &'a mut NotificationService,
        worker_controller: &'a mut WorkerController,
        command_line: &'a mut CommandLineState,
        input: &'a mut InputParser,
        show_hints: &'a mut bool,
    ) -> Self {
        Self {
            state,
            screen_stack,
            screen_manager,
            terminal,
            cfg_state,
            key_bindings,
            repo,
            notification_service,
            worker_controller,
            command_line,
            input,
            show_hints,
        }
    }

    pub async fn handle_input_command(&mut self, event: InputCommand) -> Result<ScreenState> {
        match event {
            InputCommand::Action(cmd) => self.handle_action_command(cmd).await,
            InputCommand::ModeSwitch(mode) => {
                if mode == Mode::Normal
                    && self.state.mode == Mode::Normal
                    && is_modal_screen(self.state.current_screen)
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
                *self.show_hints = !*self.show_hints;
                Ok(ScreenState::Refresh)
            }
            InputCommand::Text(text) => match self.state.mode {
                Mode::Command => self.handle_command_line_event(text).await,
                Mode::Insert => self.handle_insert_input(text).await,
                _ => Ok(ScreenState::Stay),
            },
        }
    }

    pub fn enforce_command_mode_allowed(&mut self) {
        if self.state.mode == Mode::Command && !self.command_mode_allowed() {
            self.set_mode(Mode::Normal);
        }
    }

    async fn handle_action_command(&mut self, cmd: Command) -> Result<ScreenState> {
        if let ActionId::EnterInsert(_) = cmd.action {
            self.set_mode(Mode::Insert);
        }
        if board_required_active(self.state) {
            return self.handle_board_required_action(cmd).await;
        }
        if cmd.action == ActionId::Refresh && self.is_jira_screen(self.state.current_screen) {
            return self
                .handle_sync_action(SyncAction::Pull, SyncSource::Button)
                .await;
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
        if self.state.current_screen == ScreenType::Settings {
            return self.handle_settings_command(cmd).await;
        }

        let state = self
            .with_active_screen(self.state.current_screen, |screen| {
                screen.handle_command(cmd)
            })
            .await?;
        if let ScreenState::ResolveConflictLocal(key) = &state {
            self.resolve_conflict(key.as_str(), false).await?;
            return Ok(ScreenState::Refresh);
        }
        if let ScreenState::ResolveConflictRemote(key) = &state {
            self.resolve_conflict(key.as_str(), true).await?;
            return Ok(ScreenState::Refresh);
        }
        match state {
            ScreenState::SyncNow => {
                self.enqueue_sync_now();
                return Ok(ScreenState::Refresh);
            }
            ScreenState::SyncPause => {
                self.worker_controller.pause(None);
                return Ok(ScreenState::Refresh);
            }
            ScreenState::SyncRetry => {
                self.retry_last_sync();
                return Ok(ScreenState::Refresh);
            }
            ScreenState::SyncResume => {
                self.worker_controller.resume();
                return Ok(ScreenState::Refresh);
            }
            _ => {}
        }
        self.normalize_screen_state(state, true)
    }

    async fn handle_command_line_event(&mut self, event: TextInput) -> Result<ScreenState> {
        match self.command_line.handle_event(event) {
            CommandLineOutcome::Updated => Ok(ScreenState::Refresh),
            CommandLineOutcome::Cancelled => {
                self.set_mode(Mode::Normal);
                Ok(ScreenState::Refresh)
            }
            CommandLineOutcome::Submitted(action) => {
                if matches!(action, Some(CommandLineAction::Invalid)) {
                    return self
                        .handle_command_line_action(CommandLineAction::Invalid)
                        .await;
                }
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
                let state = self
                    .handle_screen_command_line(CommandLineCommand::Write)
                    .await?;
                self.normalize_screen_state(state, false)
            }
            CommandLineAction::WriteQuit => {
                let state = self
                    .handle_screen_command_line(CommandLineCommand::WriteQuit)
                    .await?;
                self.normalize_screen_state(state, true)
            }
            CommandLineAction::WriteQuitAll => {
                if has_modal_stack(self.state.current_screen, self.screen_stack) {
                    let state = self
                        .handle_screen_command_line(CommandLineCommand::Write)
                        .await?;
                    let _ = self.normalize_screen_state(state, false)?;
                    close_all_modals_impl(self.state, self.screen_stack, self.terminal)
                } else {
                    let state = self
                        .handle_screen_command_line(CommandLineCommand::WriteQuit)
                        .await?;
                    self.normalize_screen_state(state, true)
                }
            }
            CommandLineAction::Quit => {
                if has_modal_stack(self.state.current_screen, self.screen_stack) {
                    if self.screen_stack.is_empty() && is_modal_screen(self.state.current_screen) {
                        return close_all_modals_impl(self.state, self.screen_stack, self.terminal);
                    }
                    return Ok(ScreenState::Close);
                }
                let state = self
                    .handle_screen_command_line(CommandLineCommand::Quit)
                    .await?;
                self.normalize_screen_state(state, true)
            }
            CommandLineAction::QuitAll => Ok(ScreenState::Quit),
            CommandLineAction::Sync(action) => {
                self.handle_sync_action(action, SyncSource::Manual).await
            }
            CommandLineAction::Invalid => {
                self.notification_service.push_notification(
                    "Invalid command".to_string(),
                    AppErrorLevel::Warning,
                    AppNotificationKind::System,
                );
                Ok(ScreenState::Refresh)
            }
        }
    }

    async fn handle_insert_input(&mut self, input: TextInput) -> Result<ScreenState> {
        if matches!(input, TextInput::Esc) {
            self.set_mode(Mode::Normal);
            return Ok(ScreenState::Refresh);
        }
        let key = match input {
            TextInput::Char(ch) => KeyCode::Char(ch),
            TextInput::Backspace => KeyCode::Backspace,
            TextInput::Delete => KeyCode::Delete,
            TextInput::Enter => KeyCode::Enter,
            TextInput::Tab => KeyCode::Tab,
            TextInput::Esc => KeyCode::Esc,
        };
        self.forward_raw_input(key).await
    }

    async fn handle_sync_action(
        &mut self,
        action: SyncAction,
        source: SyncSource,
    ) -> Result<ScreenState> {
        if !self.is_jira_screen(self.state.current_screen) {
            self.notification_service.set_error(AppErrorState::warning(
                "Sync is available only on Jira screens",
            ));
            return Ok(ScreenState::Refresh);
        }

        if let SyncAction::SwitchOffline = action {
            self.switch_to_offline()?;
            return Ok(ScreenState::Refresh);
        }

        let kind = match action {
            SyncAction::Pull => SyncJobKind::Pull,
            SyncAction::Push => SyncJobKind::Push,
            SyncAction::SwitchOffline => return Ok(ScreenState::Refresh),
        };
        self.worker_controller.enqueue(SyncJob::new(kind, source));
        Ok(ScreenState::Refresh)
    }

    fn enqueue_sync_now(&mut self) {
        self.worker_controller
            .enqueue(SyncJob::new(SyncJobKind::Pull, SyncSource::Manual));
        self.worker_controller
            .enqueue(SyncJob::new(SyncJobKind::Push, SyncSource::Manual));
    }

    fn retry_last_sync(&mut self) {
        self.worker_controller.resume();
        if let Some(mut job) = self.worker_controller.take_last_failed_job() {
            job.next_attempt_at = None;
            self.worker_controller.enqueue_front(job);
        }
    }

    async fn forward_raw_input(&mut self, code: KeyCode) -> Result<ScreenState> {
        let state = self
            .with_active_screen(self.state.current_screen, |screen| {
                screen.handle_command(Command {
                    action: ActionId::RawInput(code),
                    repeat: 1,
                })
            })
            .await?;
        self.normalize_screen_state(state, true)
    }

    async fn handle_board_required_action(&mut self, cmd: Command) -> Result<ScreenState> {
        match cmd.action {
            ActionId::OpenBoards => Ok(ScreenState::SwitchTo(ScreenType::BoardSelection)),
            ActionId::OpenProfiles => Ok(ScreenState::SwitchTo(ScreenType::Profiles)),
            ActionId::Quit if self.state.current_screen == ScreenType::Home => {
                Ok(ScreenState::Quit)
            }
            ActionId::GoHome => Ok(ScreenState::SwitchTo(ScreenType::Home)),
            _ => Ok(ScreenState::Stay),
        }
    }

    async fn handle_screen_command_line(&mut self, cmd: CommandLineCommand) -> Result<ScreenState> {
        self.with_active_screen(self.state.current_screen, |screen| {
            screen.handle_command_line(cmd)
        })
        .await
    }

    async fn with_active_screen<F>(&mut self, screen_type: ScreenType, f: F) -> Result<ScreenState>
    where
        F: FnOnce(&mut dyn crate::ui::screens::Screen) -> ScreenState,
    {
        let repo = self.repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot dispatch screen command")
        })?;
        let ctx = ScreenContext {
            cfg_state: self.cfg_state,
            app_state: self.state,
            repo: repo.clone(),
        };
        let screen = self
            .screen_manager
            .active_screen_mut(screen_type, ctx)
            .await?;
        Ok(f(screen))
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
                let repo = self.repo.as_ref().ok_or_else(|| {
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
                let screen = self.screen_manager.ensure_profiles(self.cfg_state);
                screen.move_up(cmd.repeat);
                return Ok(ScreenState::Refresh);
            }
            ActionId::MoveDown => {
                let screen = self.screen_manager.ensure_profiles(self.cfg_state);
                screen.move_down(cmd.repeat);
                return Ok(ScreenState::Refresh);
            }
            ActionId::MoveTop => {
                let screen = self.screen_manager.ensure_profiles(self.cfg_state);
                screen.move_top();
                return Ok(ScreenState::Refresh);
            }
            ActionId::MoveBottom => {
                let screen = self.screen_manager.ensure_profiles(self.cfg_state);
                screen.move_bottom();
                return Ok(ScreenState::Refresh);
            }
            _ => {}
        }

        let (is_empty, selected_menu, selected_profile) = {
            let screen = self.screen_manager.ensure_profiles(self.cfg_state);
            (
                screen.is_empty(),
                screen.selected_menu_id().map(str::to_owned),
                screen.selected_profile_id().map(str::to_owned),
            )
        };

        match cmd.action {
            ActionId::Confirm => {
                if is_empty {
                    if matches!(selected_menu.as_deref(), Some("quit")) {
                        return Ok(ScreenState::Quit);
                    }
                    self.start_profile_creation(ProfileEditorIntent::New);
                    return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
                }
                let Some(profile_id) = selected_profile else {
                    return Ok(ScreenState::Stay);
                };
                let mut cfg = match &*self.cfg_state {
                    AppConfigState::Loaded(cfg) => cfg.clone(),
                    AppConfigState::Missing(_) => AppConfig::default(),
                };
                if cfg.profiles.iter().any(|p| p.id == profile_id) {
                    cfg.set_active_profile(profile_id.as_str());
                    self.save_config(cfg)?;
                }
                Ok(ScreenState::Refresh)
            }
            ActionId::EditProfile => {
                if is_empty {
                    self.start_profile_creation(ProfileEditorIntent::New);
                    return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
                }
                if let (AppConfigState::Loaded(cfg), Some(id)) =
                    (&*self.cfg_state, selected_profile.as_deref())
                {
                    if cfg.profiles.iter().any(|p| p.id == id) {
                        self.start_profile_creation(ProfileEditorIntent::Edit(id.to_string()));
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
                let mut cfg = match &*self.cfg_state {
                    AppConfigState::Loaded(cfg) => cfg.clone(),
                    AppConfigState::Missing(_) => AppConfig::default(),
                };
                if cfg.remove_profile(profile_id.as_str()) {
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

    async fn handle_settings_command(&mut self, cmd: Command) -> Result<ScreenState> {
        let screen = self.screen_manager.ensure_settings(self.cfg_state);
        Ok(screen.handle_command(cmd))
    }

    fn start_profile_creation(&mut self, intent: ProfileEditorIntent) {
        self.state.profile_editor = Some(intent);
        self.screen_manager.invalidate(ScreenType::ProfileCreation);
    }

    async fn ensure_board_selection_screen(
        &mut self,
    ) -> Result<&mut crate::ui::screens::board_selection::BoardSelectionScreen> {
        if self.screen_manager.board_selection_mut().is_none() {
            let repo = self.repo.as_ref().ok_or_else(|| {
                color_eyre::eyre::eyre!("Repository not initialized: cannot open boards")
            })?;
            let ctx = ScreenContext {
                cfg_state: self.cfg_state,
                app_state: self.state,
                repo: repo.clone(),
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

    fn normalize_screen_state(
        &mut self,
        state: ScreenState,
        close_on_save: bool,
    ) -> Result<ScreenState> {
        match state {
            ScreenState::SaveProfile(profile) => {
                if let Err(err) = self.save_profile(profile) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                }
                Ok(ScreenState::Refresh)
            }
            ScreenState::SaveProfileAndClose(profile) => {
                if let Err(err) = self.save_profile(profile) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                    return Ok(ScreenState::Refresh);
                }
                if close_on_save {
                    Ok(ScreenState::Close)
                } else {
                    Ok(ScreenState::Refresh)
                }
            }
            ScreenState::ApplyTheme(theme_id) => {
                if let Err(err) = self.save_theme(theme_id.as_str()) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                }
                if let Some(screen) = self.screen_manager.settings_themes_mut() {
                    screen.set_active_theme(theme_id.as_str());
                }
                Ok(ScreenState::Refresh)
            }
            ScreenState::SaveCustomTheme(theme) => {
                if let Err(err) = self.save_custom_theme(theme) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                    return Ok(ScreenState::Refresh);
                }
                let theme_id = self.current_theme_id().to_string();
                if let Some(screen) = self.screen_manager.settings_themes_mut() {
                    screen.set_active_theme(theme_id.as_str());
                }
                self.screen_manager.invalidate(ScreenType::SettingsThemes);
                Ok(ScreenState::Refresh)
            }
            ScreenState::SaveCustomThemeAndClose(theme) => {
                if let Err(err) = self.save_custom_theme(theme) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                    return Ok(ScreenState::Refresh);
                }
                self.screen_manager.invalidate(ScreenType::SettingsThemes);
                if close_on_save {
                    Ok(ScreenState::Close)
                } else {
                    Ok(ScreenState::Refresh)
                }
            }
            ScreenState::CreateIssue(issue) => {
                if let Err(err) = self.save_issue(*issue) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                    return Ok(ScreenState::Refresh);
                }
                if close_on_save {
                    Ok(ScreenState::Close)
                } else {
                    Ok(ScreenState::Refresh)
                }
            }
            other => Ok(other),
        }
    }

    fn save_profile(&mut self, profile: ProfileConfig) -> Result<()> {
        let mut cfg = match &*self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.clone(),
            AppConfigState::Missing(_) => AppConfig::default(),
        };
        let profile_id = profile.id.clone();
        let name_lower = profile.name.to_lowercase();
        if cfg
            .profiles
            .iter()
            .any(|p| p.id != profile_id && p.name.to_lowercase() == name_lower)
        {
            return Err(eyre!(format!(
                "A profile named \"{}\" already exists",
                profile.name
            )));
        }
        cfg.upsert_profile(profile);
        cfg.set_active_profile(&profile_id);
        self.save_config(cfg)?;
        self.state.profile_editor = Some(ProfileEditorIntent::Edit(profile_id.clone()));
        if let Some(screen) = self.screen_manager.profile_creation_mut() {
            screen.set_profile_id(profile_id);
        }
        Ok(())
    }

    fn save_config(&mut self, cfg: AppConfig) -> Result<()> {
        cfg.save()?;
        *self.key_bindings = Arc::new(KeyBindings::from_config(&cfg.keybindings));
        if let Some(repo) = self.repo.as_ref() {
            *self.repo = Some(Arc::new(repo.with_profile(cfg.active_profile())?));
        } else {
            return Err(eyre!(
                "Repository not initialized: cannot refresh active profile"
            ));
        }
        *self.cfg_state = AppConfigState::Loaded(cfg);
        let selected_profile_id = self
            .screen_manager
            .profiles_mut()
            .and_then(|screen| screen.selected_profile_id().map(|id| id.to_string()));
        if let Some(screen) = self.screen_manager.profiles_mut() {
            if let AppConfigState::Loaded(cfg) = &self.cfg_state {
                screen.refresh(
                    &cfg.profiles,
                    cfg.active_profile_id.as_deref(),
                    selected_profile_id.as_deref(),
                );
            }
        }
        self.screen_manager
            .invalidate_many(&[ScreenType::Home, ScreenType::CurrentSprint]);
        Ok(())
    }

    fn save_theme(&mut self, theme_id: &str) -> Result<()> {
        let mut cfg = match &*self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.clone(),
            AppConfigState::Missing(_) => AppConfig::default(),
        };
        cfg.ui.set_theme(theme_id);
        cfg.save()?;
        *self.cfg_state = AppConfigState::Loaded(cfg);
        Ok(())
    }

    fn save_custom_theme(&mut self, mut theme: crate::config::CustomThemeConfig) -> Result<()> {
        theme.id = theme.id.to_lowercase();
        let mut cfg = match &*self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.clone(),
            AppConfigState::Missing(_) => AppConfig::default(),
        };
        if crate::ui::theme::ThemeRegistry::is_builtin_id(theme.id.as_str()) {
            return Err(eyre!(
                "Theme id \"{}\" conflicts with built-in theme",
                theme.id
            ));
        }
        if let Some(existing) = cfg.ui.custom_themes.iter_mut().find(|t| t.id == theme.id) {
            *existing = theme.clone();
        } else {
            cfg.ui.custom_themes.push(theme.clone());
        }
        cfg.ui.set_theme(theme.id.as_str());
        cfg.save()?;
        *self.cfg_state = AppConfigState::Loaded(cfg);
        Ok(())
    }

    fn current_theme_id(&self) -> &str {
        match &*self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.ui.theme.as_str(),
            AppConfigState::Missing(_) => "default",
        }
    }

    fn save_issue(&mut self, issue: crate::data::IssueSummary) -> Result<()> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot save issue")
        })?;

        // Save to database using async block
        let issue_key = issue.key.clone();
        let is_temp_key = issue_key.starts_with("TEMP-");

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // First, save the issue to the cache
                repo.cache().upsert_issues(&[issue]).await?;

                // Then, enqueue an outbox command to sync to Jira
                let change_set = serde_json::json!({
                    "action": if is_temp_key { "create" } else { "update" },
                    "key": issue_key
                })
                .to_string();

                repo.cache()
                    .enqueue_outbox(crate::data::model::OutboxCommand::issue(
                        issue_key.clone(),
                        change_set,
                    ))
                    .await
            })
        })?;

        self.notification_service.push_notification(
            format!("Issue {} created successfully", issue_key),
            AppErrorLevel::Info,
            AppNotificationKind::System,
        );

        // Invalidate relevant screens to show the new issue
        self.screen_manager.invalidate_many(&[
            ScreenType::MyIssues,
            ScreenType::SearchIssues,
            ScreenType::CurrentSprint,
        ]);

        Ok(())
    }

    async fn resolve_conflict(&mut self, key: &str, use_remote: bool) -> Result<()> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot resolve conflicts")
        })?;
        if use_remote {
            repo.resolve_conflict_use_remote(key).await?;
        } else {
            repo.resolve_conflict_use_local(key).await?;
        }

        let issues = repo.conflict_issues().await.unwrap_or_default();
        let count = issues.len();
        if self.screen_manager.conflicts_mut().is_none() {
            let ctx = ScreenContext {
                cfg_state: self.cfg_state,
                app_state: self.state,
                repo: repo.clone(),
            };
            let _ = self
                .screen_manager
                .active_screen_mut(ScreenType::Conflicts, ctx)
                .await?;
        }
        if let Some(screen) = self.screen_manager.conflicts_mut() {
            screen.set_issues(issues);
        }
        self.state.conflict_count = count;
        self.screen_manager.invalidate(ScreenType::Home);
        Ok(())
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
            ActionId::OpenProfiles => {
                if self.state.current_screen == ScreenType::Settings {
                    Some(ScreenState::SwitchTo(ScreenType::Profiles))
                } else {
                    None
                }
            }
            ActionId::OpenMyIssues => Some(ScreenState::SwitchTo(ScreenType::MyIssues)),
            ActionId::OpenSearchIssues => Some(ScreenState::SwitchTo(ScreenType::SearchIssues)),
            ActionId::OpenNewIssue => Some(ScreenState::SwitchTo(ScreenType::NewIssue)),
            ActionId::OpenBoards => Some(ScreenState::SwitchTo(ScreenType::BoardSelection)),
            ActionId::OpenSettings => Some(ScreenState::SwitchTo(ScreenType::Settings)),
            ActionId::OpenSyncStatus => Some(ScreenState::SwitchTo(ScreenType::SyncStatus)),
            _ => None,
        }
    }

    fn switch_to_offline(&mut self) -> Result<()> {
        let mut cfg = match &*self.cfg_state {
            AppConfigState::Loaded(cfg) => cfg.clone(),
            AppConfigState::Missing(_) => AppConfig::default(),
        };
        if let Some(profile) = cfg.active_profile_mut() {
            profile.set_sync_mode(SyncMode::Cache);
            let name = profile.name.clone();
            self.save_config(cfg)?;
            self.notification_service.push_notification(
                format!("Profile \"{name}\" switched to offline mode"),
                AppErrorLevel::Info,
                AppNotificationKind::System,
            );
        }
        Ok(())
    }

    fn is_jira_screen(&self, screen: ScreenType) -> bool {
        matches!(
            screen,
            ScreenType::CurrentSprint
                | ScreenType::MyIssues
                | ScreenType::SearchIssues
                | ScreenType::NewIssue
        )
    }
}
