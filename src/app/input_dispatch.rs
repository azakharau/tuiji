use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;

use super::*;
use crate::app::{
    command_router::{CommandRouter, CommandRouterDeps},
    input::{CommandResolver, is_question_mark},
    state::Mode,
};
use crate::ui::screens::ScreenState;

impl App {
    pub(super) async fn handle_input(&mut self, key: KeyEvent) -> Result<ScreenState> {
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
        let mut router = CommandRouter::new(CommandRouterDeps {
            state: &mut self.state,
            screen_stack: &mut self.screen_stack,
            screen_manager: &mut self.screen_manager,
            terminal: &mut self.terminal,
            cfg_state: &mut self.cfg_state,
            key_bindings: &mut self.key_bindings,
            repo: &mut self.repo,
            notification_service: &mut self.notification_service,
            worker_controller: &mut self.worker_controller,
            command_line: &mut self.command_line,
            input: &mut self.input,
            show_hints: &mut self.show_hints,
        });
        router.handle_input_command(cmd).await
    }

    pub(super) fn enforce_command_mode_allowed(&mut self) {
        if self.state.mode == Mode::Command && matches!(self.state.current_screen, ScreenType::Home)
        {
            self.state.mode = Mode::Normal;
            self.input.clear_pending();
            self.command_line.stop();
        }
    }
}
