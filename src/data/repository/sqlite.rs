use std::path::PathBuf;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    config::resolve_config_dir,
    data::{
        model::{
            BoardColumn, BoardConfig, BoardSummary, ColumnStatusRef, Estimation, IssueComment,
            IssueSummary, OutboxCommand, SyncLogEntry, SyncLogFilter, SyncState,
        },
        repository::{CommandRepository, QueryRepository},
    },
};

mod boards;
mod conflicts;
mod outbox;
mod query;
mod seeds;
mod writes;

#[derive(Debug, Clone)]
pub struct SqliteRepositoryConfig {
    pub db_path: PathBuf,
}

#[derive(Clone)]
pub struct SqliteRepository {
    cfg: SqliteRepositoryConfig,
    pool: SqlitePool,
    profile_id: String,
}

pub struct SprintUpsert<'a> {
    pub board_id: u64,
    pub sprint_id: u64,
    pub name: &'a str,
    pub state: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub complete_date: Option<i64>,
}

impl SqliteRepository {
    pub fn default_db_path() -> PathBuf {
        resolve_config_dir().join("tuiji.db")
    }

    pub async fn connect(cfg: SqliteRepositoryConfig, profile_id: String) -> Result<Self> {
        if let Some(parent) = cfg.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let options = SqliteConnectOptions::new()
            .filename(&cfg.db_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self {
            cfg,
            pool,
            profile_id,
        })
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.cfg.db_path
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn with_profile(&self, profile_id: String) -> Self {
        Self {
            cfg: self.cfg.clone(),
            pool: self.pool.clone(),
            profile_id,
        }
    }

    fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    pub async fn log_sync_event(
        &self,
        direction: &str,
        status: &str,
        error: Option<&str>,
        profile_id: Option<&str>,
    ) -> Result<()> {
        self.log_sync_event_impl(direction, status, error, profile_id)
            .await
    }

    pub async fn seed_mock_data_if_empty(&self) -> Result<Option<u64>> {
        self.seed_mock_data_if_empty_impl().await
    }

    pub async fn sync_log(&self, limit: usize, filter: SyncLogFilter) -> Result<Vec<SyncLogEntry>> {
        self.sync_log_impl(limit, filter).await
    }

    async fn fetch_comments_for_issue(&self, issue_key: &str) -> Result<Vec<IssueComment>> {
        self.fetch_comments_for_issue_impl(issue_key).await
    }

    pub async fn fetch_issue(&self, issue_key: &str) -> Result<Option<IssueSummary>> {
        self.fetch_issue_impl(issue_key).await
    }
}

#[async_trait]
impl QueryRepository for SqliteRepository {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig> {
        self.board_config_impl(board_id).await
    }

    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>> {
        self.current_sprint_issues_impl(board_id).await
    }

    async fn sync_state(&self) -> Result<SyncState> {
        self.load_sync_state().await
    }
}

#[async_trait]
impl CommandRepository for SqliteRepository {
    async fn upsert_issues(&self, issues: &[IssueSummary]) -> Result<()> {
        self.upsert_issues_impl(issues).await
    }

    async fn set_sync_state(&self, state: SyncState) -> Result<()> {
        self.persist_sync_state(state).await
    }

    async fn enqueue_outbox(&self, command: OutboxCommand) -> Result<()> {
        self.enqueue_outbox_impl(command).await
    }
}

fn ts_to_system_time(value: Option<i64>) -> Option<std::time::SystemTime> {
    value.map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64))
}

fn system_time_to_ts(value: Option<std::time::SystemTime>) -> Option<i64> {
    value.map(|ts| {
        ts.duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    })
}

fn current_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn parse_json_array(value: Option<String>) -> Vec<String> {
    value
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn serialize_json_array(value: &[String]) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        serde_json::to_string(value).ok()
    }
}

#[derive(Debug, Clone)]
pub struct OutboxRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub change_set: String,
    pub attempts: i64,
}
