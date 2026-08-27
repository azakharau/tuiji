use super::*;
use crate::app::services::sync as sync_service;

impl<'a> CommandRouter<'a> {
    pub(super) async fn handle_sync_action(
        &mut self,
        action: SyncAction,
        source: SyncSource,
    ) -> Result<ScreenState> {
        sync_service::handle_sync_action(
            action,
            source,
            self.state.current_screen,
            sync_service::SyncActionDeps {
                notification_service: self.notification_service,
                worker_controller: self.worker_controller,
            },
        )
        .await
    }

    pub(super) fn enqueue_sync_now(&mut self) {
        sync_service::enqueue_sync_now(self.worker_controller);
    }

    pub(super) async fn retry_last_sync(&mut self) -> Result<()> {
        sync_service::retry_last_sync(self.repo, self.worker_controller).await
    }

    pub(super) async fn resolve_conflict(&mut self, key: &str, use_remote: bool) -> Result<()> {
        sync_service::resolve_conflict(
            key,
            use_remote,
            self.state,
            self.cfg_state,
            self.repo,
            self.screen_manager,
        )
        .await
    }
}
