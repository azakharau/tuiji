use crossterm::event::KeyCode;

use super::*;
use crate::app::command_router::policies::{
    screen_transition::ScreenTransitionPolicy, state_mutation::StateMutationPolicy,
};
use crate::app::navigation::{close_all_modals_impl, has_modal_stack, is_modal_screen};

impl<'a> CommandRouter<'a> {
    pub(super) async fn handle_command_line_event(
        &mut self,
        event: TextInput,
    ) -> Result<ScreenState> {
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

    pub(super) async fn handle_command_line_action(
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
                    close_all_modals_impl(self.state, self.screen_stack, self.screen_manager)
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
                        return close_all_modals_impl(
                            self.state,
                            self.screen_stack,
                            self.screen_manager,
                        );
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

    pub(super) async fn handle_insert_input(&mut self, input: TextInput) -> Result<ScreenState> {
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

    pub(super) async fn forward_raw_input(&mut self, code: KeyCode) -> Result<ScreenState> {
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

    pub(super) async fn handle_screen_command_line(
        &mut self,
        cmd: CommandLineCommand,
    ) -> Result<ScreenState> {
        self.with_active_screen(self.state.current_screen, |screen| {
            screen.handle_command_line(cmd)
        })
        .await
    }

    pub(super) fn command_mode_allowed(&self) -> bool {
        ScreenTransitionPolicy::command_mode_allowed(self.state.current_screen)
    }

    pub(super) fn set_mode(&mut self, mode: Mode) {
        let command_mode_allowed = self.command_mode_allowed();
        StateMutationPolicy::apply_mode(
            self.state,
            self.input,
            self.command_line,
            mode,
            command_mode_allowed,
        );
    }
}
