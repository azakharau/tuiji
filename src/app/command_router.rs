use std::sync::Arc;

use color_eyre::Result;
use ratatui::DefaultTerminal;

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

pub struct CommandRouterDeps<'a> {
    pub state: &'a mut AppState,
    pub screen_stack: &'a mut Vec<ScreenType>,
    pub screen_manager: &'a mut ScreenManager,
    pub terminal: &'a mut DefaultTerminal,
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
            terminal: deps.terminal,
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
}
