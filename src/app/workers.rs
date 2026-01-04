use std::time::Duration;

use color_eyre::Result;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle, time};

use crate::{
    app::event::{AppEvent, WorkerEvent},
    client::jira::JiraClient,
    config::AppConfig,
    data::repository::CommandRepository,
    data::{IssueSummary, OutboxCommand, SqliteRepository},
};

const PULL_INTERVAL_SECS: u64 = 60;
const PUSH_INTERVAL_SECS: u64 = 20;
const OUTBOX_BATCH_SIZE: i64 = 20;
const MAX_OUTBOX_ATTEMPTS: i64 = 5;

pub fn spawn_sync_pull_worker(
    tx: UnboundedSender<AppEvent>,
    cfg: AppConfig,
    db: SqliteRepository,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let Some(profile) = cfg.active_profile() else {
            let _ = tx.send(AppEvent::Worker(WorkerEvent::Notification(
                "sync: no active profile configured".to_string(),
            )));
            return;
        };
        let jira = match JiraClient::new(
            profile.jira.base_url.as_str(),
            profile.jira.username.as_str(),
            profile.jira.api_token.as_str(),
        ) {
            Ok(client) => client,
            Err(err) => {
                let _ = tx.send(AppEvent::Worker(WorkerEvent::Notification(format!(
                    "sync: jira client init failed: {err}"
                ))));
                return;
            }
        };

        let mut ticker = time::interval(Duration::from_secs(PULL_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            if let Err(err) = sync_once(&jira, &db).await {
                let _ = tx.send(AppEvent::Worker(WorkerEvent::Notification(format!(
                    "sync: {err}"
                ))));
            }
        }
    })
}

pub fn spawn_sync_push_worker(
    tx: UnboundedSender<AppEvent>,
    cfg: AppConfig,
    db: SqliteRepository,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let Some(profile) = cfg.active_profile() else {
            let _ = tx.send(AppEvent::Worker(WorkerEvent::Notification(
                "push: no active profile configured".to_string(),
            )));
            return;
        };
        let jira = match JiraClient::new(
            profile.jira.base_url.as_str(),
            profile.jira.username.as_str(),
            profile.jira.api_token.as_str(),
        ) {
            Ok(client) => client,
            Err(err) => {
                let _ = tx.send(AppEvent::Worker(WorkerEvent::Notification(format!(
                    "push: jira client init failed: {err}"
                ))));
                return;
            }
        };

        let mut ticker = time::interval(Duration::from_secs(PUSH_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            if let Err(err) = push_once(&jira, &db).await {
                let _ = tx.send(AppEvent::Worker(WorkerEvent::Notification(format!(
                    "push: {err}"
                ))));
            }
        }
    })
}

async fn sync_once(jira: &JiraClient, db: &SqliteRepository) -> Result<()> {
    let board_ids = db.selected_board_ids().await?;
    for board_id in board_ids {
        sync_board(jira, db, board_id).await?;
    }
    Ok(())
}

async fn sync_board(jira: &JiraClient, db: &SqliteRepository, board_id: u64) -> Result<()> {
    let board = jira.get_board(board_id).await?;
    let location_json = board.location.as_ref().map(|loc| {
        serde_json::json!({
            "project_id": loc.project_id,
            "user_id": loc.user_id,
            "user_account_id": loc.user_account_id,
            "display_name": loc.display_name,
            "project_name": loc.project_name,
            "project_key": loc.project_key,
            "project_type_key": loc.project_type_key,
            "name": loc.name,
        })
        .to_string()
    });
    db.upsert_board(board_id, &board.name, &board.type_name, location_json)
        .await?;

    let config = jira.get_board_config(board_id).await?;
    db.upsert_board_config(board_id, &config).await?;
    db.replace_board_columns(board_id, &config.columns).await?;

    let sprint = jira.get_current_sprint(board_id).await?;
    db.upsert_sprint(
        board_id,
        sprint.id,
        &sprint.name,
        sprint.state.clone(),
        sprint.start_date.map(|d| d.unix_timestamp()),
        sprint.end_date.map(|d| d.unix_timestamp()),
        sprint.complete_date.map(|d| d.unix_timestamp()),
    )
    .await?;

    let issues = jira.get_sprint_issues(sprint.id).await?;
    let summaries = map_issues(issues, Some(sprint.id as i64));
    db.upsert_issues(summaries).await?;
    Ok(())
}

fn map_issues(issues: Vec<gouqi::Issue>, sprint_id: Option<i64>) -> Vec<IssueSummary> {
    issues
        .into_iter()
        .map(|issue| {
            let key = issue.key.to_string();
            let project_key = key.split('-').next().map(|v| v.to_string());
            let summary = issue.summary().unwrap_or_default();
            let epic = None;
            let status = match issue.status() {
                Some(st) => st.name.to_uppercase(),
                None => "TODO".to_string(),
            };
            let issue_type = match issue.issue_type() {
                Some(it) => it.name,
                None => "Task".to_string(),
            };
            let assignee = match issue.assignee() {
                Some(user) => user.display_name,
                None => "Unassigned".to_string(),
            };
            let priority = match issue.priority() {
                Some(pr) => pr.name,
                None => "Medium".to_string(),
            };
            let story_points = None;
            let updated_at = issue.updated().map(|ts| {
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.unix_timestamp() as u64)
            });

            IssueSummary {
                key,
                summary,
                epic,
                status,
                issue_type,
                assignee,
                priority,
                story_points,
                project_key,
                sprint_id,
                updated_at,
            }
        })
        .collect()
}

async fn push_once(_jira: &JiraClient, db: &SqliteRepository) -> Result<()> {
    let items = db.fetch_pending_outbox(OUTBOX_BATCH_SIZE).await?;
    for item in items {
        if item.attempts >= MAX_OUTBOX_ATTEMPTS {
            db.mark_outbox_failed(&item.id, "max attempts reached")
                .await?;
            continue;
        }

        db.mark_outbox_processing(&item.id).await?;
        let cmd = decode_outbox_command(&item.command_type, &item.payload_json);
        match cmd {
            Ok(_cmd) => {
                db.mark_outbox_failed(&item.id, "jira push not implemented")
                    .await?;
            }
            Err(err) => {
                db.mark_outbox_failed(&item.id, &err.to_string()).await?;
            }
        }
    }
    Ok(())
}

fn decode_outbox_command(command_type: &str, payload_json: &str) -> Result<OutboxCommand> {
    let payload: serde_json::Value = serde_json::from_str(payload_json)?;
    match command_type {
        "CreateIssue" => Ok(OutboxCommand::CreateIssue {
            summary: payload
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }),
        "UpdateIssue" => Ok(OutboxCommand::UpdateIssue {
            key: payload
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }),
        other => Err(color_eyre::eyre::eyre!(format!(
            "unknown outbox command: {other}"
        ))),
    }
}
