use crate::{
    data::SyncLogFilter,
    ui::interaction::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::SyncStatusState;

pub struct SyncStatusController;

impl SyncStatusController {
    pub fn handle_command(_state: &mut SyncStatusState, command: Command) -> ScreenState {
        match command.action {
            ActionId::SyncNow => ScreenState::SyncNow,
            ActionId::SyncPause => ScreenState::SyncPause,
            ActionId::SyncRetry => ScreenState::SyncRetry,
            ActionId::SyncResume => ScreenState::SyncResume,
            ActionId::FilterAll => {
                _state.set_filter(SyncLogFilter::All);
                ScreenState::Refresh
            }
            ActionId::FilterPull => {
                _state.set_filter(SyncLogFilter::Pull);
                ScreenState::Refresh
            }
            ActionId::FilterPush => {
                _state.set_filter(SyncLogFilter::Push);
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}
