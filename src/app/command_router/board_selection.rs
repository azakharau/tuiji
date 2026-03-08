use super::*;
use crate::app::services::boards;

impl<'a> CommandRouter<'a> {
    pub(super) async fn handle_board_selection_command(
        &mut self,
        cmd: Command,
    ) -> Result<ScreenState> {
        boards::handle_board_selection_command(
            cmd,
            self.state,
            self.screen_stack,
            self.screen_manager,
            self.cfg_state,
            self.repo,
        )
        .await
    }
}
