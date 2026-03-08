use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use super::*;
use crate::app::{
    event::{AppEvent, WorkerEvent},
    screen_manager::ScreenContext,
    state::ScreenType,
    worker_controller::SyncJobEvent,
};
use crate::contracts::{
    error::AppErrorLevel,
    notification::AppNotificationKind,
    sync::{SyncJob, SyncJobKind},
};
use crate::data::{ConflictRepository, SyncExecutor};

pub(super) async fn handle_worker_event(app: &mut App, msg: WorkerEvent) -> bool {
    match msg {
        WorkerEvent::JiraUpdated => true,
        WorkerEvent::Notification(message) => {
            app.notification_service.push_notification(
                message,
                AppErrorLevel::Info,
                AppNotificationKind::System,
            );
            true
        }
        WorkerEvent::SyncCompleted(job) => {
            let kind = job.kind;
            app.worker_controller
                .handle_worker_event(SyncJobEvent::Completed(job));
            app.screen_manager.invalidate(app.state.current_screen);
            if kind == SyncJobKind::Pull {
                refresh_conflicts_after_pull(app).await;
            }
            true
        }
        WorkerEvent::SyncFailed { job, error } => {
            app.worker_controller
                .handle_worker_event(SyncJobEvent::Failed { job, error });
            true
        }
    }
}

pub(super) fn start_next_job(app: &mut App, tx: &UnboundedSender<AppEvent>) {
    let Some(job) = app.worker_controller.start_next() else {
        return;
    };
    let Some(repo) = app.repo.clone() else {
        app.worker_controller
            .handle_worker_event(SyncJobEvent::Failed {
                job,
                error: "Repository not initialized: cannot sync".to_string(),
            });
        return;
    };

    let sync_repo: std::sync::Arc<dyn SyncExecutor> = repo;
    spawn_sync_job(tx.clone(), sync_repo, job);
}

async fn refresh_conflicts_after_pull(app: &mut App) {
    let Some(repo) = app.repo.clone() else {
        return;
    };
    let Ok(count) = repo.conflict_count().await else {
        return;
    };

    let prev_count = app.state.conflict_count;
    if count > prev_count {
        app.notification_service.push_notification(
            format!("Conflicts detected ({count})"),
            AppErrorLevel::Warning,
            AppNotificationKind::System,
        );
        if let Ok(issues) = repo.conflict_issues().await {
            let ctx = ScreenContext {
                cfg_state: &app.cfg_state,
                app_state: &app.state,
                repo: repo.clone(),
            };
            if app
                .screen_manager
                .active_screen_mut(ScreenType::Conflicts, ctx)
                .await
                .is_ok()
                && let Some(screen) = app.screen_manager.conflicts_mut()
            {
                screen.set_issues(issues);
            }
        }
        if app.state.current_screen != ScreenType::Conflicts {
            app.screen_stack.push(app.state.current_screen);
            app.state.current_screen = ScreenType::Conflicts;
        }
    }
    app.state.conflict_count = count;
    app.screen_manager.invalidate(ScreenType::Home);
}

fn spawn_sync_job(
    tx: UnboundedSender<AppEvent>,
    repo: std::sync::Arc<dyn SyncExecutor>,
    job: SyncJob,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let result = match job.kind {
            SyncJobKind::Pull => repo.sync_pull().await,
            SyncJobKind::Push => repo.sync_push().await,
        };
        let event = match result {
            Ok(()) => WorkerEvent::SyncCompleted(job),
            Err(err) => WorkerEvent::SyncFailed {
                job,
                error: err.to_string(),
            },
        };
        let _ = tx.send(AppEvent::Worker(event));
    })
}
