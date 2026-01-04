use std::path::PathBuf;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    client::jira::{BoardColumn, BoardConfig, ColumnStatusRef, Estimation},
    config::resolve_config_dir,
    data::{
        model::{BoardSummary, IssueSummary, OutboxCommand, SyncState},
        repository::{CommandRepository, QueryRepository},
    },
};

#[derive(Debug, Clone)]
pub struct SqliteRepositoryConfig {
    pub db_path: PathBuf,
}

#[derive(Clone)]
pub struct SqliteRepository {
    cfg: SqliteRepositoryConfig,
    pool: SqlitePool,
}

impl SqliteRepository {
    pub fn default_db_path() -> PathBuf {
        resolve_config_dir().join("tuiji.db")
    }

    pub async fn connect(cfg: SqliteRepositoryConfig) -> Result<Self> {
        if let Some(parent) = cfg.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let options = SqliteConnectOptions::new()
            .filename(&cfg.db_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { cfg, pool })
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.cfg.db_path
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn seed_mock_data_if_empty(&self) -> Result<Option<u64>> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM boards")
            .fetch_one(&self.pool)
            .await?;
        let count: i64 = row.get("count");
        if count > 0 {
            return Ok(None);
        }

        let board_id: u64 = 175;
        let now = current_ts();
        let sprint_id: i64 = 1001;

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO boards (id, name, type_name, location_json, updated_at)
            VALUES (?, ?, ?, NULL, ?)
            "#,
        )
        .bind(board_id as i64)
        .bind("Demo Board")
        .bind("scrum")
        .bind(now)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO board_config (board_id, estimation_type, estimation_field_id, raw_json, updated_at)
            VALUES (?, ?, ?, NULL, ?)
            "#,
        )
        .bind(board_id as i64)
        .bind("StoryPoints")
        .bind("customfield_10002")
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let columns = vec![
            ("TODO", vec!["1", "2"]),
            ("IN PROGRESS", vec!["3"]),
            ("DONE", vec!["4"]),
        ];
        for (idx, (name, statuses)) in columns.into_iter().enumerate() {
            let status_ids_json = serde_json::to_string(&statuses)?;
            sqlx::query(
                r#"
                INSERT INTO board_columns (board_id, position, name, status_ids_json)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(board_id as i64)
            .bind(idx as i64)
            .bind(name)
            .bind(status_ids_json)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO sprints (id, board_id, name, state, start_date, end_date, complete_date, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, NULL, ?)
            "#,
        )
        .bind(sprint_id)
        .bind(board_id as i64)
        .bind("Sprint 1")
        .bind("active")
        .bind(now - 7 * 24 * 3600)
        .bind(now + 7 * 24 * 3600)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let issues = vec![
            (
                "DEMO-1",
                "Setup local cache",
                "TODO",
                "Task",
                "Medium",
                "Alex",
                Some(3.0),
            ),
            (
                "DEMO-2",
                "Fix login error",
                "IN PROGRESS",
                "Bug",
                "High",
                "Maria",
                Some(5.0),
            ),
            (
                "DEMO-3",
                "Ship MVP",
                "DONE",
                "Story",
                "Medium",
                "Unassigned",
                Some(8.0),
            ),
        ];

        for (key, summary, status, issue_type, priority, assignee, story_points) in issues {
            sqlx::query(
                r#"
                INSERT INTO issues (
                    key,
                    summary,
                    status,
                    issue_type,
                    priority,
                    assignee,
                    epic,
                    story_points,
                    sprint_id,
                    project_key,
                    updated_at,
                    synced_at,
                    raw_json
                ) VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, NULL)
                "#,
            )
            .bind(key)
            .bind(summary)
            .bind(status)
            .bind(issue_type)
            .bind(priority)
            .bind(assignee)
            .bind(story_points)
            .bind(sprint_id)
            .bind("DEMO")
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO selected_boards (board_id, is_default, selected_at)
            VALUES (?, 1, ?)
            "#,
        )
        .bind(board_id as i64)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Some(board_id))
    }

    pub async fn upsert_board(
        &self,
        board_id: u64,
        name: &str,
        type_name: &str,
        location_json: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO boards (id, name, type_name, location_json, updated_at)
            VALUES (?, ?, ?, ?, strftime('%s','now'))
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                type_name = excluded.type_name,
                location_json = excluded.location_json,
                updated_at = strftime('%s','now')
            "#,
        )
        .bind(board_id as i64)
        .bind(name)
        .bind(type_name)
        .bind(location_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_board_config(&self, board_id: u64, config: &BoardConfig) -> Result<()> {
        let (estimation_type, estimation_field_id) = match &config.estimation {
            Estimation::StoryPoints(field_id) => ("StoryPoints", field_id.as_str()),
            Estimation::DateBased(field_id) => ("DateBased", field_id.as_str()),
        };
        sqlx::query(
            r#"
            INSERT INTO board_config (board_id, estimation_type, estimation_field_id, raw_json, updated_at)
            VALUES (?, ?, ?, NULL, strftime('%s','now'))
            ON CONFLICT(board_id) DO UPDATE SET
                estimation_type = excluded.estimation_type,
                estimation_field_id = excluded.estimation_field_id,
                updated_at = strftime('%s','now')
            "#,
        )
        .bind(board_id as i64)
        .bind(estimation_type)
        .bind(estimation_field_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn replace_board_columns(
        &self,
        board_id: u64,
        columns: &[BoardColumn],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM board_columns WHERE board_id = ?")
            .bind(board_id as i64)
            .execute(&mut *tx)
            .await?;
        for (idx, column) in columns.iter().enumerate() {
            let status_ids = column
                .statuses
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<String>>();
            let status_ids_json = serde_json::to_string(&status_ids)?;
            sqlx::query(
                r#"
                INSERT INTO board_columns (board_id, position, name, status_ids_json)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(board_id as i64)
            .bind(idx as i64)
            .bind(&column.name)
            .bind(status_ids_json)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_sprint(
        &self,
        board_id: u64,
        sprint_id: u64,
        name: &str,
        state: Option<String>,
        start_date: Option<i64>,
        end_date: Option<i64>,
        complete_date: Option<i64>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sprints (id, board_id, name, state, start_date, end_date, complete_date, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%s','now'))
            ON CONFLICT(id) DO UPDATE SET
                board_id = excluded.board_id,
                name = excluded.name,
                state = excluded.state,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                complete_date = excluded.complete_date,
                updated_at = strftime('%s','now')
            "#,
        )
        .bind(sprint_id as i64)
        .bind(board_id as i64)
        .bind(name)
        .bind(state)
        .bind(start_date)
        .bind(end_date)
        .bind(complete_date)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn selected_board_ids(&self) -> Result<Vec<u64>> {
        let rows = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            ORDER BY selected_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut ids = Vec::with_capacity(rows.len());
        for row in rows {
            let board_id: i64 = row.get("board_id");
            ids.push(board_id as u64);
        }
        Ok(ids)
    }

    pub async fn list_boards(&self) -> Result<Vec<BoardSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, type_name
            FROM boards
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut boards = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            boards.push(BoardSummary {
                id: id as u64,
                name: row.get("name"),
                type_name: row.get("type_name"),
            });
        }
        Ok(boards)
    }

    pub async fn default_board_id(&self) -> Result<Option<u64>> {
        let row = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            WHERE is_default = 1
            ORDER BY selected_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let board_id: i64 = row.get("board_id");
            return Ok(Some(board_id as u64));
        }

        let fallback = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            ORDER BY selected_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(fallback.map(|row| row.get::<i64, _>("board_id") as u64))
    }

    pub async fn set_selected_board(&self, board_id: u64, is_default: bool) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        if is_default {
            sqlx::query(
                r#"
                UPDATE selected_boards
                SET is_default = 0
                WHERE is_default = 1
                "#,
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO selected_boards (board_id, is_default, selected_at)
            VALUES (?, ?, strftime('%s','now'))
            ON CONFLICT(board_id) DO UPDATE SET
                is_default = excluded.is_default,
                selected_at = strftime('%s','now')
            "#,
        )
        .bind(board_id as i64)
        .bind(if is_default { 1 } else { 0 })
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn fetch_pending_outbox(&self, limit: i64) -> Result<Vec<OutboxRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, command_type, payload_json, attempts
            FROM outbox
            WHERE status = 'pending'
            ORDER BY created_at
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(OutboxRecord {
                id: row.get("id"),
                command_type: row.get("command_type"),
                payload_json: row.get("payload_json"),
                attempts: row.get("attempts"),
            });
        }
        Ok(items)
    }

    pub async fn mark_outbox_processing(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'processing',
                attempts = attempts + 1,
                updated_at = strftime('%s','now')
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_outbox_done(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'done',
                updated_at = strftime('%s','now')
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_outbox_failed(&self, id: &str, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'failed',
                last_error = ?,
                updated_at = strftime('%s','now')
            WHERE id = ?
            "#,
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl QueryRepository for SqliteRepository {
    async fn board_config(&self, _board_id: u64) -> Result<BoardConfig> {
        let record = sqlx::query(
            r#"
            SELECT estimation_type, estimation_field_id
            FROM board_config
            WHERE board_id = ?
            "#,
        )
        .bind(_board_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        let record =
            record.ok_or_else(|| eyre!("Board config not found for board {}", _board_id))?;
        let estimation_type: String = record.get("estimation_type");
        let estimation_field_id: String = record.get("estimation_field_id");
        let estimation = match estimation_type.as_str() {
            "StoryPoints" | "storypoints" | "story_points" | "story" => {
                Estimation::StoryPoints(estimation_field_id)
            }
            "DateBased" | "datebased" | "days" | "date" => {
                Estimation::DateBased(estimation_field_id)
            }
            _ => Estimation::StoryPoints(estimation_field_id),
        };

        let column_rows = sqlx::query(
            r#"
            SELECT name, status_ids_json
            FROM board_columns
            WHERE board_id = ?
            ORDER BY position
            "#,
        )
        .bind(_board_id as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut columns = Vec::with_capacity(column_rows.len());
        for row in column_rows {
            let name: String = row.get("name");
            let status_ids_json: String = row.get("status_ids_json");
            let status_ids: Vec<String> =
                serde_json::from_str(&status_ids_json).unwrap_or_default();
            let statuses = status_ids
                .into_iter()
                .map(|id| ColumnStatusRef {
                    id,
                    self_link: String::new(),
                })
                .collect();
            let mut column = BoardColumn { name, statuses };
            column.name = column.name.to_uppercase();
            columns.push(column);
        }

        Ok(BoardConfig {
            columns,
            estimation,
        })
    }

    async fn current_sprint_issues(&self, _board_id: u64) -> Result<Vec<IssueSummary>> {
        let sprint = sqlx::query(
            r#"
            SELECT id
            FROM sprints
            WHERE board_id = ? AND state = 'active'
            ORDER BY start_date DESC
            LIMIT 1
            "#,
        )
        .bind(_board_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        let Some(sprint) = sprint else {
            return Ok(Vec::new());
        };

        let rows = sqlx::query(
            r#"
            SELECT key, summary, status, issue_type, priority, assignee, epic, story_points,
                   sprint_id, project_key, updated_at
            FROM issues
            WHERE sprint_id = ?
            "#,
        )
        .bind(sprint.get::<i64, _>("id"))
        .fetch_all(&self.pool)
        .await?;

        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            let key: String = row.get("key");
            let project_key: Option<String> = row.get("project_key");
            issues.push(IssueSummary {
                key: key.clone(),
                summary: row.get("summary"),
                epic: row.get("epic"),
                status: row.get("status"),
                issue_type: row.get("issue_type"),
                assignee: row.get("assignee"),
                priority: row.get("priority"),
                story_points: row.get("story_points"),
                project_key: project_key.or_else(|| key.split('-').next().map(|v| v.to_string())),
                sprint_id: row.get("sprint_id"),
                updated_at: ts_to_system_time(row.get("updated_at")),
            });
        }

        Ok(issues)
    }

    async fn sync_state(&self) -> Result<SyncState> {
        let row = sqlx::query(
            r#"
            SELECT last_full_sync, last_pull, last_push
            FROM sync_state
            WHERE id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(SyncState::default());
        };

        Ok(SyncState {
            last_full_sync: ts_to_system_time(row.get("last_full_sync")),
            last_pull: ts_to_system_time(row.get("last_pull")),
            last_push: ts_to_system_time(row.get("last_push")),
        })
    }
}

#[async_trait]
impl CommandRepository for SqliteRepository {
    async fn upsert_issues(&self, issues: Vec<IssueSummary>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        for issue in issues {
            let project_key = issue
                .project_key
                .clone()
                .or_else(|| issue.key.split('-').next().map(|v| v.to_string()));
            let updated_at = system_time_to_ts(issue.updated_at);
            sqlx::query(
                r#"
                INSERT INTO issues (
                    key,
                    summary,
                    status,
                    issue_type,
                    priority,
                    assignee,
                    epic,
                    story_points,
                    sprint_id,
                    project_key,
                    updated_at,
                    synced_at,
                    raw_json
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), NULL)
                ON CONFLICT(key) DO UPDATE SET
                    summary = excluded.summary,
                    status = excluded.status,
                    issue_type = excluded.issue_type,
                    priority = excluded.priority,
                    assignee = excluded.assignee,
                    epic = excluded.epic,
                    story_points = excluded.story_points,
                    sprint_id = excluded.sprint_id,
                    project_key = excluded.project_key,
                    updated_at = excluded.updated_at,
                    synced_at = strftime('%s','now')
                "#,
            )
            .bind(&issue.key)
            .bind(&issue.summary)
            .bind(&issue.status)
            .bind(&issue.issue_type)
            .bind(&issue.priority)
            .bind(&issue.assignee)
            .bind(&issue.epic)
            .bind(issue.story_points)
            .bind(issue.sprint_id)
            .bind(project_key)
            .bind(updated_at)
            .execute(&mut *tx)
            .await?;

            let snapshot_at = updated_at.unwrap_or(now_ts);
            sqlx::query(
                r#"
                INSERT INTO issue_history (
                    issue_key,
                    snapshot_at,
                    summary,
                    status,
                    issue_type,
                    priority,
                    assignee,
                    epic,
                    story_points,
                    sprint_id,
                    raw_json
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
                "#,
            )
            .bind(&issue.key)
            .bind(snapshot_at)
            .bind(&issue.summary)
            .bind(&issue.status)
            .bind(&issue.issue_type)
            .bind(&issue.priority)
            .bind(&issue.assignee)
            .bind(&issue.epic)
            .bind(issue.story_points)
            .bind(issue.sprint_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn set_sync_state(&self, state: SyncState) -> Result<()> {
        let last_full_sync = system_time_to_ts(state.last_full_sync);
        let last_pull = system_time_to_ts(state.last_pull);
        let last_push = system_time_to_ts(state.last_push);
        sqlx::query(
            r#"
            INSERT INTO sync_state (id, last_full_sync, last_pull, last_push, updated_at)
            VALUES (1, ?, ?, ?, strftime('%s','now'))
            ON CONFLICT(id) DO UPDATE SET
                last_full_sync = excluded.last_full_sync,
                last_pull = excluded.last_pull,
                last_push = excluded.last_push,
                updated_at = strftime('%s','now')
            "#,
        )
        .bind(last_full_sync)
        .bind(last_pull)
        .bind(last_push)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn enqueue_outbox(&self, command: OutboxCommand) -> Result<()> {
        let (command_type, payload_json) = match command {
            OutboxCommand::CreateIssue { summary } => (
                "CreateIssue",
                serde_json::to_string(&serde_json::json!({ "summary": summary }))?,
            ),
            OutboxCommand::UpdateIssue { key } => (
                "UpdateIssue",
                serde_json::to_string(&serde_json::json!({ "key": key }))?,
            ),
        };
        let id = uuid::Uuid::now_v7().to_string();
        sqlx::query(
            r#"
            INSERT INTO outbox (id, command_type, payload_json, status, attempts, created_at, updated_at)
            VALUES (?, ?, ?, 'pending', 0, strftime('%s','now'), strftime('%s','now'))
            "#,
        )
        .bind(id)
        .bind(command_type)
        .bind(payload_json)
        .execute(&self.pool)
        .await?;
        Ok(())
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

#[derive(Debug, Clone)]
pub struct OutboxRecord {
    pub id: String,
    pub command_type: String,
    pub payload_json: String,
    pub attempts: i64,
}
