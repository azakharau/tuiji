use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use log::error;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle, time};

use crate::app::screen_manager::ScreenContext;
use crate::app::state::ScreenType;
use crate::app::{
    ActionOutcome, App,
    event::{AppEvent, InputEvent, NotificationEvent, SystemEvent, UiEvent, WorkerEvent},
    input::is_question_mark,
    navigation::NavigationController,
    notification::AppNotificationKind,
    worker_controller::{SyncJobEvent, SyncJobKind},
};
use crate::data::AppRepository;

pub async fn run(app: &mut App) -> Result<()> {
    app.init_db().await?;
    app.terminal.clear()?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _input = spawn_input_listener(tx.clone());
    let _tick = spawn_tick(tx.clone(), Duration::from_millis(250));

    // Initial paint.
    app.init_start_screen();
    app.render().await?;

    while let Some(event) = rx.recv().await {
        let mut render_requested = false;

        match event {
            AppEvent::Input(InputEvent::Key(key)) => {
                if app.worker_controller.is_paused() {
                    render_requested = handle_sync_modal_input(app, key);
                } else {
                    let action = app.handle_input(key).await?;
                    let outcome = {
                        let mut nav = NavigationController::new(
                            &mut app.state,
                            &mut app.screen_stack,
                            &mut app.screen_manager,
                            &mut app.terminal,
                            &app.cfg_state,
                            &app.key_bindings,
                        );
                        nav.apply_action(action)?
                    };
                    app.enforce_command_mode_allowed();
                    match outcome {
                        ActionOutcome::Continue { render } => render_requested |= render,
                        ActionOutcome::Quit => {
                            break;
                        }
                    }
                }
            }
            AppEvent::System(SystemEvent::Tick) => {
                render_requested |= app.notification_service.tick();
            }
            AppEvent::Worker(msg) => {
                render_requested |= handle_worker_event(app, msg).await;
            }
            AppEvent::Ui(ui_event) => {
                if let UiEvent::Error(err) = ui_event {
                    app.notification_service
                        .set_error(crate::app::error::AppErrorState::error(err));
                    render_requested = true;
                }
            }
            AppEvent::Notification(notification) => {
                let NotificationEvent::Message(msg) = notification;
                app.notification_service.push_notification(
                    msg,
                    crate::app::error::AppErrorLevel::Info,
                    AppNotificationKind::Reminder,
                );
                render_requested = true;
            }

            AppEvent::Nav(_) | AppEvent::Repo(_) => {}
        }

        if !app.worker_controller.is_paused() {
            start_next_job(app, &tx);
        }

        if render_requested {
            app.render().await?;
        }
    }

    Ok(())
}

async fn handle_worker_event(app: &mut App, msg: WorkerEvent) -> bool {
    match msg {
        WorkerEvent::JiraUpdated => true,
        WorkerEvent::Notification(message) => {
            app.notification_service.push_notification(
                message,
                crate::app::error::AppErrorLevel::Info,
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
                if let Some(repo) = app.repo.clone() {
                    if let Ok(count) = repo.conflict_count().await {
                        let prev_count = app.state.conflict_count;
                        if count > prev_count {
                            app.notification_service.push_notification(
                                format!("Conflicts detected ({count})"),
                                crate::app::error::AppErrorLevel::Warning,
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
                                {
                                    if let Some(screen) = app.screen_manager.conflicts_mut() {
                                        screen.set_issues(issues);
                                    }
                                }
                            }
                            if app.state.current_screen != ScreenType::Conflicts {
                                app.screen_stack.push(app.state.current_screen);
                                app.state.current_screen = ScreenType::Conflicts;
                            }
                        }
                        app.state.conflict_count = count;
                        app.screen_manager
                            .invalidate(crate::app::state::ScreenType::Home);
                    }
                }
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

fn handle_sync_modal_input(app: &mut App, key: KeyEvent) -> bool {
    if is_question_mark(&key) {
        return true;
    }
    match key.code {
        KeyCode::Char('r') => {
            app.worker_controller.resume();
            if let Some(job) = app.worker_controller.take_last_failed_job() {
                let mut job = job;
                job.next_attempt_at = None;
                app.worker_controller.enqueue_front(job);
            }
            true
        }
        KeyCode::Char('q') => {
            app.worker_controller.stop();
            true
        }
        _ => false,
    }
}

fn start_next_job(app: &mut App, tx: &UnboundedSender<AppEvent>) {
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

    spawn_sync_job(tx.clone(), repo, job);
}

fn spawn_sync_job(
    tx: UnboundedSender<AppEvent>,
    repo: std::sync::Arc<crate::data::RepositoryHub>,
    job: crate::app::worker_controller::SyncJob,
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

fn spawn_input_listener(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(event) = reader.next().await {
            match event {
                Ok(Event::Key(key)) => {
                    let _ = tx.send(AppEvent::Input(InputEvent::Key(key)));
                }
                Ok(_) => {}
                Err(err) => {
                    error!("Event stream error: {}", err);
                    break;
                }
            }
        }
    })
}

fn spawn_tick(tx: UnboundedSender<AppEvent>, period: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = time::interval(period);
        loop {
            ticker.tick().await;
            if tx.send(AppEvent::System(SystemEvent::Tick)).is_err() {
                break;
            }
        }
    })
}
