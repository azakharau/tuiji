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

pub async fn handle_sync_action(
    action: SyncAction,
    source: SyncSource,
    current_screen: ScreenType,
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
    if !is_jira_screen(current_screen) {
        notification_service.set_error(AppErrorState::warning(
            "Sync is available only on Jira screens",
        ));
        return Ok(ScreenState::Refresh);
    }

    if let SyncAction::SwitchOffline = action {
        if let Some(name) =
            configuration::switch_to_offline(cfg_state, key_bindings, repo, screen_manager)?
        {
            notification_service.push_notification(
                format!("Profile \"{name}\" switched to offline mode"),
                AppErrorLevel::Info,
                AppNotificationKind::System,
            );
        }
        return Ok(ScreenState::Refresh);
    }

    let kind = match action {
        SyncAction::Pull => SyncJobKind::Pull,
        SyncAction::Push => SyncJobKind::Push,
        SyncAction::SwitchOffline => return Ok(ScreenState::Refresh),
    };
    worker_controller.enqueue(SyncJob::new(kind, source));
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
    if use_remote {
        repo.resolve_conflict_use_remote(key).await?;
    } else {
        repo.resolve_conflict_use_local(key).await?;
    }

    let issues = repo.conflict_issues().await.unwrap_or_default();
    let count = issues.len();
    if screen_manager.conflicts_mut().is_none() {
        let ctx = ScreenContext {
            cfg_state,
            app_state: state,
            repo: repo.clone(),
        };
        let _ = screen_manager
            .active_screen_mut(ScreenType::Conflicts, ctx)
            .await?;
    }
    if let Some(screen) = screen_manager.conflicts_mut() {
        screen.set_issues(issues);
    }
    state.conflict_count = count;
    screen_manager.invalidate(ScreenType::Home);
    Ok(())
}

fn is_jira_screen(screen: ScreenType) -> bool {
    matches!(
        screen,
        ScreenType::CurrentSprint
            | ScreenType::MyIssues
            | ScreenType::SearchIssues
            | ScreenType::NewIssue
    )
}
