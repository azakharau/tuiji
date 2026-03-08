use color_eyre::eyre::Result;

use super::{AppRenderer, RenderState};
use crate::app::{
    screen_manager::{ScreenContext, ScreenManager},
    state::ScreenType,
};
use crate::data::SyncStatusRepository;

impl AppRenderer {
    pub async fn prepare(
        screen_manager: &mut ScreenManager,
        state: &RenderState<'_>,
    ) -> Result<()> {
        for screen_type in state.render_stack.iter() {
            let ctx = ScreenContext {
                cfg_state: state.cfg_state,
                app_state: state.app_state,
                repo: state.repo.clone(),
            };
            let _ = screen_manager.active_screen_mut(screen_type, ctx).await?;
        }

        if should_refresh_sync_status(state) {
            refresh_sync_status(screen_manager, state).await;
        }

        Ok(())
    }
}

fn should_refresh_sync_status(state: &RenderState<'_>) -> bool {
    state
        .render_stack
        .iter()
        .any(|screen| screen == ScreenType::SyncStatus)
}

async fn refresh_sync_status(screen_manager: &mut ScreenManager, state: &RenderState<'_>) {
    let Some(screen) = screen_manager.sync_status_mut() else {
        return;
    };

    screen.set_snapshot(state.sync_status.clone());
    let filter = screen.filter();
    if let Ok(entries) = state.repo.sync_log(10, filter).await {
        screen.set_log(entries);
    }
}
