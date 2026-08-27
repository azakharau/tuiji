use super::*;
use crate::app::command_router::policies::{
    effects::{ScreenEffect, ScreenEffectPolicy, SyncControl},
    screen_transition::ScreenTransitionPolicy,
};
use crate::{
    app::{
        error::AppErrorState,
        worker_controller::{SyncJob, SyncJobKind},
    },
    data::repository::MutationRepository,
};

impl<'a> CommandRouter<'a> {
    pub(super) async fn handle_action_command(&mut self, cmd: Command) -> Result<ScreenState> {
        if let ActionId::EnterInsert(_) = cmd.action {
            self.set_mode(Mode::Insert);
        }
        if board_required_active(self.state) {
            return self.handle_board_required_action(cmd).await;
        }
        if cmd.action == ActionId::Refresh
            && ScreenTransitionPolicy::is_jira_screen(self.state.current_screen)
        {
            return self
                .handle_sync_action(SyncAction::Pull, SyncSource::Button)
                .await;
        }
        if let Some(nav) = self.map_global_action(&cmd) {
            return Ok(nav);
        }
        if self.state.current_screen == ScreenType::BoardSelection {
            return self.handle_board_selection_command(cmd).await;
        }
        if self.state.current_screen == ScreenType::Profiles {
            return self.handle_profiles_command(cmd).await;
        }
        if self.state.current_screen == ScreenType::Settings {
            return self.handle_settings_command(cmd).await;
        }

        let state = self
            .with_active_screen(self.state.current_screen, |screen| {
                screen.handle_command(cmd)
            })
            .await?;
        match ScreenEffectPolicy::classify(&state) {
            ScreenEffect::ResolveConflict { key, use_remote } => {
                self.resolve_conflict(key, use_remote).await?;
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::Mutate(mutation) => {
                self.set_mode(Mode::Normal);
                let Some(repo) = self.repo.as_ref() else {
                    self.notification_service.set_error(AppErrorState::error(
                        "Repository not initialized: cannot apply issue mutation",
                    ));
                    return Ok(ScreenState::Refresh);
                };
                if let Err(err) = repo.apply_mutation(mutation.clone()).await {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                    return Ok(ScreenState::Refresh);
                }
                self.worker_controller
                    .enqueue(SyncJob::new(SyncJobKind::Push, SyncSource::Manual));
                self.screen_manager.invalidate(self.state.current_screen);
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::OpenInBrowser(url) => {
                if let Err(err) = crate::app::services::browser::open_url(url) {
                    self.notification_service
                        .set_error(AppErrorState::error(err.to_string()));
                }
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::RunSearch(jql) => {
                self.set_mode(Mode::Normal);
                self.state.search_issues_query = Some(jql.to_owned());
                self.screen_manager.invalidate(ScreenType::SearchIssues);
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::Sync(SyncControl::Now) => {
                self.enqueue_sync_now();
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::Sync(SyncControl::Pause) => {
                self.worker_controller.pause(None);
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::Sync(SyncControl::Retry) => {
                self.retry_last_sync().await?;
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::Sync(SyncControl::Resume) => {
                self.worker_controller.resume();
                return Ok(ScreenState::Refresh);
            }
            ScreenEffect::None => {}
        }
        self.normalize_screen_state(state, true)
    }

    pub(super) async fn with_active_screen<F>(
        &mut self,
        screen_type: ScreenType,
        f: F,
    ) -> Result<ScreenState>
    where
        F: FnOnce(&mut dyn crate::ui::screens::Screen) -> ScreenState,
    {
        let repo = self.repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot dispatch screen command")
        })?;
        let ctx = ScreenContext {
            cfg_state: self.cfg_state,
            app_state: self.state,
            repo: repo.clone(),
        };
        let screen = self
            .screen_manager
            .active_screen_mut(screen_type, ctx)
            .await?;
        Ok(f(screen))
    }
}
