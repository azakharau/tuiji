use std::sync::Arc;

use color_eyre::Result;

use crate::{
    app::{
        AppState,
        error::AppErrorState,
        input::SyncAction,
        key_handlers::KeyBindings,
        notification::AppNotificationKind,
        notification_service::NotificationService,
        screen_manager::{ScreenContext, ScreenManager},
        screen_policy,
        state::ScreenType,
        worker_controller::{SyncJob, SyncJobKind, SyncSource, WorkerController},
    },
    config::AppConfigState,
    contracts::error::AppErrorLevel,
    data::{ConflictRepository, RepositoryHub},
    ui::screens::ScreenState,
};

use super::configuration;

pub struct SyncActionDeps<'a> {
    pub cfg_state: &'a mut AppConfigState,
    pub key_bindings: &'a mut Arc<KeyBindings>,
    pub repo: &'a mut Option<Arc<RepositoryHub>>,
    pub screen_manager: &'a mut ScreenManager,
    pub notification_service: &'a mut NotificationService,
    pub worker_controller: &'a mut WorkerController,
}

enum SyncActionRequest {
    RejectNonJira,
    SwitchOffline,
    Queue(SyncJobKind),
}

pub async fn handle_sync_action(
    action: SyncAction,
    source: SyncSource,
    current_screen: ScreenType,
    deps: SyncActionDeps<'_>,
) -> Result<ScreenState> {
    apply_sync_action_request(classify_sync_action(action, current_screen), source, deps)
}

fn classify_sync_action(action: SyncAction, current_screen: ScreenType) -> SyncActionRequest {
    if !screen_policy::is_jira_screen(current_screen) {
        return SyncActionRequest::RejectNonJira;
    }

    match action {
        SyncAction::Pull => SyncActionRequest::Queue(SyncJobKind::Pull),
        SyncAction::Push => SyncActionRequest::Queue(SyncJobKind::Push),
        SyncAction::SwitchOffline => SyncActionRequest::SwitchOffline,
    }
}

fn apply_sync_action_request(
    request: SyncActionRequest,
    source: SyncSource,
    deps: SyncActionDeps<'_>,
) -> Result<ScreenState> {
    let SyncActionDeps {
        cfg_state,
        key_bindings,
        repo,
        screen_manager,
        notification_service,
        worker_controller,
    } = deps;
    match request {
        SyncActionRequest::RejectNonJira => {
            notification_service.set_error(AppErrorState::warning(
                "Sync is available only on Jira screens",
            ));
        }
        SyncActionRequest::SwitchOffline => {
            if let Some(name) =
                configuration::switch_to_offline(cfg_state, key_bindings, repo, screen_manager)?
            {
                notification_service.push_notification(
                    format!("Profile \"{name}\" switched to offline mode"),
                    AppErrorLevel::Info,
                    AppNotificationKind::System,
                );
            }
        }
        SyncActionRequest::Queue(kind) => worker_controller.enqueue(SyncJob::new(kind, source)),
    }

    Ok(ScreenState::Refresh)
}

pub fn enqueue_sync_now(worker_controller: &mut WorkerController) {
    worker_controller.enqueue(SyncJob::new(SyncJobKind::Pull, SyncSource::Manual));
    worker_controller.enqueue(SyncJob::new(SyncJobKind::Push, SyncSource::Manual));
}

pub fn retry_last_sync(worker_controller: &mut WorkerController) {
    worker_controller.resume();
    if let Some(mut job) = worker_controller.take_last_failed_job() {
        job.next_attempt_at = None;
        worker_controller.enqueue_front(job);
    }
}

pub async fn resolve_conflict(
    key: &str,
    use_remote: bool,
    state: &mut AppState,
    cfg_state: &AppConfigState,
    repo: &Option<Arc<RepositoryHub>>,
    screen_manager: &mut ScreenManager,
) -> Result<()> {
    let repo = repo.as_ref().ok_or_else(|| {
        color_eyre::eyre::eyre!("Repository not initialized: cannot resolve conflicts")
    })?;

    apply_conflict_resolution(repo, key, use_remote).await?;
    refresh_conflict_resolution(state, cfg_state, repo.clone(), screen_manager).await
}

async fn apply_conflict_resolution(
    repo: &RepositoryHub,
    key: &str,
    use_remote: bool,
) -> Result<()> {
    if use_remote {
        repo.resolve_conflict_use_remote(key).await?;
    } else {
        repo.resolve_conflict_use_local(key).await?;
    }

    Ok(())
}

async fn refresh_conflict_resolution(
    state: &mut AppState,
    cfg_state: &AppConfigState,
    repo: Arc<RepositoryHub>,
    screen_manager: &mut ScreenManager,
) -> Result<()> {
    let issues = repo.conflict_issues().await.unwrap_or_default();
    let count = issues.len();
    ensure_conflicts_screen_loaded(screen_manager, cfg_state, state, repo.clone()).await?;
    if let Some(screen) = screen_manager.conflicts_mut() {
        screen.set_issues(issues);
    }
    state.conflict_count = count;
    screen_manager.invalidate(ScreenType::Home);
    Ok(())
}

async fn ensure_conflicts_screen_loaded(
    screen_manager: &mut ScreenManager,
    cfg_state: &AppConfigState,
    state: &mut AppState,
    repo: Arc<RepositoryHub>,
) -> Result<()> {
    if screen_manager.conflicts_mut().is_none() {
        let ctx = ScreenContext {
            cfg_state,
            app_state: state,
            repo,
        };
        let _ = screen_manager
            .active_screen_mut(ScreenType::Conflicts, ctx)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SyncActionRequest, classify_sync_action};
    use crate::app::{input::SyncAction, state::ScreenType, worker_controller::SyncJobKind};

    #[test]
    fn sync_action_should_reject_non_jira_screens() {
        assert!(matches!(
            classify_sync_action(SyncAction::Pull, ScreenType::Home),
            SyncActionRequest::RejectNonJira
        ));
    }

    #[test]
    fn sync_action_should_queue_pull_on_jira_screens() {
        assert!(matches!(
            classify_sync_action(SyncAction::Pull, ScreenType::CurrentSprint),
            SyncActionRequest::Queue(SyncJobKind::Pull)
        ));
    }

    #[test]
    fn sync_action_should_preserve_switch_offline_on_jira_screens() {
        assert!(matches!(
            classify_sync_action(SyncAction::SwitchOffline, ScreenType::MyIssues),
            SyncActionRequest::SwitchOffline
        ));
    }
}
