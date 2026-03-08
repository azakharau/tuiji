use super::*;
use crate::app::services::screen_state as screen_state_service;

impl<'a> CommandRouter<'a> {
    pub(super) fn normalize_screen_state(
        &mut self,
        state: ScreenState,
        close_on_save: bool,
    ) -> Result<ScreenState> {
        screen_state_service::normalize_screen_state(
            state,
            close_on_save,
            screen_state_service::NormalizeStateDeps {
                state: self.state,
                cfg_state: self.cfg_state,
                key_bindings: self.key_bindings,
                repo: self.repo,
                screen_manager: self.screen_manager,
                notification_service: self.notification_service,
            },
        )
    }
}
