use super::*;
use crate::app::command_router::policies::screen_transition::ScreenTransitionPolicy;

impl<'a> CommandRouter<'a> {
    pub(super) async fn handle_board_required_action(
        &mut self,
        cmd: Command,
    ) -> Result<ScreenState> {
        Ok(ScreenTransitionPolicy::board_required_action(
            self.state.current_screen,
            cmd.action,
        ))
    }

    pub(super) fn map_global_action(&mut self, cmd: &Command) -> Option<ScreenState> {
        ScreenTransitionPolicy::global_action(self.state.current_screen, cmd.action)
    }
}
