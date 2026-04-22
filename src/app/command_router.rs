use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::KeyCode;

use crate::{
    app::{
        AppState,
        error::AppErrorLevel,
        input::{
            CommandLineAction, CommandLineOutcome, CommandLineState, InputCommand, InputParser,
            SyncAction, TextInput,
        },
        key_handlers::{ActionId, Command, KeyBindings},
        navigation::{board_required_active, is_modal_screen},
        notification::AppNotificationKind,
        notification_service::NotificationService,
        screen_manager::{ScreenContext, ScreenManager},
        state::{Mode, ScreenType},
        worker_controller::{SyncSource, WorkerController},
    },
    config::AppConfigState,
    data::RepositoryHub,
    ui::screens::{CommandLineCommand, ScreenState},
};
use policies::screen_transition::{ModeSwitchDecision, ScreenTransitionPolicy};

mod action_dispatch;
mod board_selection;
mod input_flow;
mod navigation;
mod policies;
mod profiles;
mod state_normalization;
mod sync;

pub struct CommandRouter<'a> {
    state: &'a mut AppState,
    screen_stack: &'a mut Vec<ScreenType>,
    screen_manager: &'a mut ScreenManager,
    cfg_state: &'a mut AppConfigState,
    key_bindings: &'a mut Arc<KeyBindings>,
    repo: &'a mut Option<Arc<RepositoryHub>>,
    notification_service: &'a mut NotificationService,
    worker_controller: &'a mut WorkerController,
    command_line: &'a mut CommandLineState,
    input: &'a mut InputParser,
    show_hints: &'a mut bool,
}

pub struct CommandRouterDeps<'a> {
    pub state: &'a mut AppState,
    pub screen_stack: &'a mut Vec<ScreenType>,
    pub screen_manager: &'a mut ScreenManager,
    pub cfg_state: &'a mut AppConfigState,
    pub key_bindings: &'a mut Arc<KeyBindings>,
    pub repo: &'a mut Option<Arc<RepositoryHub>>,
    pub notification_service: &'a mut NotificationService,
    pub worker_controller: &'a mut WorkerController,
    pub command_line: &'a mut CommandLineState,
    pub input: &'a mut InputParser,
    pub show_hints: &'a mut bool,
}

impl<'a> CommandRouter<'a> {
    pub fn new(deps: CommandRouterDeps<'a>) -> Self {
        Self {
            state: deps.state,
            screen_stack: deps.screen_stack,
            screen_manager: deps.screen_manager,
            cfg_state: deps.cfg_state,
            key_bindings: deps.key_bindings,
            repo: deps.repo,
            notification_service: deps.notification_service,
            worker_controller: deps.worker_controller,
            command_line: deps.command_line,
            input: deps.input,
            show_hints: deps.show_hints,
        }
    }

    pub async fn handle_input_command(&mut self, event: InputCommand) -> Result<ScreenState> {
        match event {
            InputCommand::Action(cmd) => self.handle_action_command(cmd).await,
            InputCommand::ModeSwitch(mode) => {
                if self.should_forward_escape_to_screen(mode) {
                    let state = self.forward_raw_input(KeyCode::Esc).await?;
                    if state != ScreenState::Stay {
                        return Ok(state);
                    }
                }
                match ScreenTransitionPolicy::mode_switch(
                    self.state.mode,
                    mode,
                    self.state.current_screen,
                    is_modal_screen(self.state.current_screen),
                ) {
                    ModeSwitchDecision::CloseModal => Ok(ScreenState::Close),
                    ModeSwitchDecision::Reject => Ok(ScreenState::Stay),
                    ModeSwitchDecision::Apply(mode) => {
                        self.set_mode(mode);
                        Ok(ScreenState::Refresh)
                    }
                }
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

    fn should_forward_escape_to_screen(&self, mode: Mode) -> bool {
        self.state.mode == Mode::Normal
            && mode == Mode::Normal
            && !is_modal_screen(self.state.current_screen)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use super::*;
    use crate::{
        ConfigError,
        app::{
            AppState,
            input::{CommandLineState, InputCommand, InputParser},
            key_handlers::KeyBindings,
            notification_service::NotificationService,
            screen_manager::ScreenManager,
            worker_controller::WorkerController,
        },
        config::{AppConfigState, KeyBindingsConfig, UiConfig},
        data::{RepositoryHub, SqliteRepositoryConfig},
    };

    #[tokio::test]
    async fn normal_mode_escape_should_close_new_issue_overlay_before_global_mode_switch() {
        let mut state = AppState {
            current_screen: ScreenType::NewIssue,
            ..AppState::default()
        };
        let mut screen_stack = Vec::new();
        let mut screen_manager = ScreenManager::default();
        let mut cfg_state = AppConfigState::Missing(ConfigError::MissingField(
            "test",
            PathBuf::from("/tmp/tuiji-test-config.toml"),
        ));
        let mut key_bindings = Arc::new(KeyBindings::from_config(&KeyBindingsConfig::default()));
        let db_path = std::env::temp_dir().join(format!(
            "tuiji-command-router-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        let repository = RepositoryHub::connect(SqliteRepositoryConfig { db_path }, None)
            .await
            .expect("connect sqlite repository for router test");
        let mut repo = Some(Arc::new(repository));
        let mut notification_service = NotificationService::from_ui(&UiConfig {
            theme: "default".to_string(),
            custom_themes: Vec::new(),
            screen_cache_ttl_seconds: 60,
            notification_ttl_seconds: 5,
            notification_stack_limit: 5,
            error_ttl_seconds: 6,
        });
        let mut worker_controller = WorkerController::default();
        let mut command_line = CommandLineState::default();
        let mut input = InputParser::default();
        let mut show_hints = false;

        let mut router = CommandRouter::new(CommandRouterDeps {
            state: &mut state,
            screen_stack: &mut screen_stack,
            screen_manager: &mut screen_manager,
            cfg_state: &mut cfg_state,
            key_bindings: &mut key_bindings,
            repo: &mut repo,
            notification_service: &mut notification_service,
            worker_controller: &mut worker_controller,
            command_line: &mut command_line,
            input: &mut input,
            show_hints: &mut show_hints,
        });

        let _ = router
            .handle_action_command(Command {
                action: ActionId::Confirm,
                repeat: 1,
            })
            .await
            .expect("open issue form popup");

        let state = router
            .handle_input_command(InputCommand::ModeSwitch(Mode::Normal))
            .await
            .expect("handle normal mode escape");

        assert_eq!(state, ScreenState::Refresh);

        let post_escape = router
            .handle_action_command(Command {
                action: ActionId::Quit,
                repeat: 1,
            })
            .await
            .expect("issue form should handle quit after escape");

        assert_eq!(post_escape, ScreenState::Stay);
    }
}
