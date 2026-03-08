use super::*;
use crate::app::services::profiles as profiles_service;

impl<'a> CommandRouter<'a> {
    pub(super) async fn handle_profiles_command(&mut self, cmd: Command) -> Result<ScreenState> {
        profiles_service::handle_profiles_command(
            cmd,
            self.state,
            self.cfg_state,
            self.key_bindings,
            self.repo,
            self.screen_manager,
        )
    }

    pub(super) async fn handle_settings_command(&mut self, cmd: Command) -> Result<ScreenState> {
        profiles_service::handle_settings_command(cmd, self.cfg_state, self.screen_manager)
    }
}
