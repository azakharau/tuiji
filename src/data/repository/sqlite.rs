use std::path::PathBuf;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    client::jira::{BoardColumn, BoardConfig, ColumnStatusRef, Estimation},
    config::resolve_config_dir,
    data::{
        model::{
            BoardSummary, IssueComment, IssueSummary, OutboxCommand, SyncLogEntry, SyncLogFilter,
            SyncState,
        },
        repository::{CommandRepository, QueryRepository},
    },
};

const OUTBOX_BATCH_SIZE: i64 = 20;
const MAX_OUTBOX_ATTEMPTS: i64 = 5;

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
        sqlx::query(
            r#"
            INSERT INTO sync_log (direction, status, error, profile_id, created_at)
            VALUES (?, ?, ?, ?, strftime('%s','now'))
            "#,
        )
        .bind(direction)
        .bind(status)
        .bind(error)
        .bind(profile_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn seed_mock_data_if_empty(&self) -> Result<Option<u64>> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM boards WHERE profile_id = ?")
            .bind(self.profile_id())
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
            INSERT INTO boards (id, name, type_name, location_json, updated_at, profile_id)
            VALUES (?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(board_id as i64)
        .bind("Demo Board")
        .bind("scrum")
        .bind(now)
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO board_config (board_id, estimation_type, estimation_field_id, raw_json, updated_at, profile_id)
            VALUES (?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(board_id as i64)
        .bind("StoryPoints")
        .bind("customfield_10002")
        .bind(now)
        .bind(self.profile_id())
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
                INSERT INTO board_columns (board_id, position, name, status_ids_json, profile_id)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(board_id as i64)
            .bind(idx as i64)
            .bind(name)
            .bind(status_ids_json)
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO sprints (id, board_id, name, state, start_date, end_date, complete_date, updated_at, profile_id)
            VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(sprint_id)
        .bind(board_id as i64)
        .bind("Sprint 1")
        .bind("active")
        .bind(now - 7 * 24 * 3600)
        .bind(now + 7 * 24 * 3600)
        .bind(now)
        .bind(self.profile_id())
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
                    raw_json,
                    profile_id,
                    description,
                    reporter,
                    creator,
                    created_at,
                    resolution_date,
                    resolution,
                    labels,
                    fix_versions,
                    parent_key,
                    environment,
                    time_estimate,
                    time_spent,
                    time_remaining,
                    custom_fields
                ) VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, NULL, ?, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL)
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
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO selected_boards (board_id, is_default, selected_at, profile_id)
            VALUES (?, 1, ?, ?)
            "#,
        )
        .bind(board_id as i64)
        .bind(now)
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Some(board_id))
    }

    pub async fn sync_log(&self, limit: usize, filter: SyncLogFilter) -> Result<Vec<SyncLogEntry>> {
        let mut sql = String::from(
            r#"
            SELECT direction,
                   status,
                   error,
                   datetime(created_at, 'unixepoch') as created_at
            FROM sync_log
            WHERE profile_id = ?
            "#,
        );
        if !matches!(filter, SyncLogFilter::All) {
            sql.push_str(" AND direction = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?");

        let mut query = sqlx::query(&sql).bind(self.profile_id());
        if !matches!(filter, SyncLogFilter::All) {
            let direction = match filter {
                SyncLogFilter::Pull => "pull",
                SyncLogFilter::Push => "push",
                SyncLogFilter::All => "pull",
            };
            query = query.bind(direction);
        }
        query = query.bind(limit as i64);

        let rows = query.fetch_all(&self.pool).await?;
        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let direction: String = row.get("direction");
            let status: String = row.get("status");
            let error: Option<String> = row.get("error");
            let created_at: String = row.get("created_at");
            entries.push(SyncLogEntry {
                direction,
                status,
                error,
                created_at,
            });
        }
        Ok(entries)
    }

    async fn fetch_comments_for_issue(&self, issue_key: &str) -> Result<Vec<IssueComment>> {
        let rows = sqlx::query(
            r#"
            SELECT id, author, body, created_at, updated_at, dirty, conflict, remote_snapshot
            FROM issue_comments
            WHERE issue_key = ?
              AND profile_id = ?
            ORDER BY created_at
            "#,
        )
        .bind(issue_key)
        .bind(self.profile_id())
        .fetch_all(&self.pool)
        .await?;
        let mut comments = Vec::with_capacity(rows.len());
        for row in rows {
            let created_at: Option<String> = row.get("created_at");
            let updated_at: Option<String> = row.get("updated_at");
            comments.push(IssueComment {
                id: row.get("id"),
                issue_key: issue_key.to_string(),
                author: row.get("author"),
                body: row.get("body"),
                created_at,
                updated_at,
                dirty: row.get::<i64, _>("dirty") != 0,
                conflict: row.get::<i64, _>("conflict") != 0,
                remote_snapshot: row.get("remote_snapshot"),
            });
        }
        Ok(comments)
    }

    pub async fn fetch_issue(&self, issue_key: &str) -> Result<Option<IssueSummary>> {
        let row = sqlx::query(
            r#"
            SELECT key, summary, status, issue_type, priority, assignee, epic, story_points,
                   sprint_id, project_key, updated_at, dirty, conflict, remote_snapshot,
                   description, reporter, creator, created_at, resolution_date, resolution,
                   labels, fix_versions, parent_key, environment,
                   time_estimate, time_spent, time_remaining, custom_fields
            FROM issues
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(issue_key)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let key: String = row.get("key");
        let project_key: Option<String> = row.get("project_key");
        let comments = self.fetch_comments_for_issue(&key).await?;
        Ok(Some(IssueSummary {
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
            comments,
            dirty: row.get::<i64, _>("dirty") != 0,
            conflict: row.get::<i64, _>("conflict") != 0,
            remote_snapshot: row.get("remote_snapshot"),

            // New fields
            description: row.get("description"),
            reporter: row.get("reporter"),
            creator: row.get("creator"),
            created_at: ts_to_system_time(row.get("created_at")),
            resolution_date: ts_to_system_time(row.get("resolution_date")),
            resolution: row.get("resolution"),
            labels: parse_json_array(row.get("labels")),
            fix_versions: parse_json_array(row.get("fix_versions")),
            parent_key: row.get("parent_key"),
            environment: row.get("environment"),
            time_estimate: row.get("time_estimate"),
            time_spent: row.get("time_spent"),
            time_remaining: row.get("time_remaining"),
            custom_fields: row.get("custom_fields"),
        }))
    }

    pub async fn list_conflict_issues(&self) -> Result<Vec<IssueSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT key, summary, status, issue_type, priority, assignee, epic, story_points,
                   sprint_id, project_key, updated_at, dirty, conflict, remote_snapshot,
                   description, reporter, creator, created_at, resolution_date, resolution,
                   labels, fix_versions, parent_key, environment,
                   time_estimate, time_spent, time_remaining, custom_fields
            FROM issues
            WHERE profile_id = ?
              AND (
                conflict = 1 OR key IN (
                    SELECT issue_key
                    FROM issue_comments
                    WHERE profile_id = ? AND conflict = 1
                )
              )
            ORDER BY key
            "#,
        )
        .bind(self.profile_id())
        .bind(self.profile_id())
        .fetch_all(&self.pool)
        .await?;

        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            let key: String = row.get("key");
            let project_key: Option<String> = row.get("project_key");
            let comments = self.fetch_comments_for_issue(&key).await?;
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
                comments,
                dirty: row.get::<i64, _>("dirty") != 0,
                conflict: row.get::<i64, _>("conflict") != 0,
                remote_snapshot: row.get("remote_snapshot"),

                // New fields
                description: row.get("description"),
                reporter: row.get("reporter"),
                creator: row.get("creator"),
                created_at: ts_to_system_time(row.get("created_at")),
                resolution_date: ts_to_system_time(row.get("resolution_date")),
                resolution: row.get("resolution"),
                labels: parse_json_array(row.get("labels")),
                fix_versions: parse_json_array(row.get("fix_versions")),
                parent_key: row.get("parent_key"),
                environment: row.get("environment"),
                time_estimate: row.get("time_estimate"),
                time_spent: row.get("time_spent"),
                time_remaining: row.get("time_remaining"),
                custom_fields: row.get("custom_fields"),
            });
        }
        Ok(issues)
    }

    pub async fn count_conflicts(&self) -> Result<usize> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT key) as count
            FROM issues
            WHERE profile_id = ?
              AND (
                conflict = 1 OR key IN (
                    SELECT issue_key
                    FROM issue_comments
                    WHERE profile_id = ? AND conflict = 1
                )
              )
            "#,
        )
        .bind(self.profile_id())
        .bind(self.profile_id())
        .fetch_one(&self.pool)
        .await?;
        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    pub async fn mark_issue_conflict(&self, key: &str, remote_snapshot: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET conflict = 1,
                remote_snapshot = ?
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(remote_snapshot)
        .bind(key)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn clear_issue_conflict(&self, key: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET conflict = 0,
                remote_snapshot = NULL
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_comment_conflict(&self, id: &str, remote_snapshot: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issue_comments
            SET conflict = 1,
                remote_snapshot = ?
            WHERE id = ? AND profile_id = ?
            "#,
        )
        .bind(remote_snapshot)
        .bind(id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn clear_comment_conflict(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issue_comments
            SET conflict = 0,
                remote_snapshot = NULL
            WHERE id = ? AND profile_id = ?
            "#,
        )
        .bind(id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn resolve_issue_use_local(&self, key: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            UPDATE issues
            SET conflict = 0,
                remote_snapshot = NULL
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            UPDATE issue_comments
            SET conflict = 0,
                remote_snapshot = NULL
            WHERE issue_key = ? AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn resolve_issue_use_remote(
        &self,
        key: &str,
        remote: &crate::data::diff::IssueSnapshot,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            UPDATE issues
            SET summary = ?,
                status = ?,
                issue_type = ?,
                priority = ?,
                assignee = ?,
                epic = ?,
                story_points = ?,
                sprint_id = ?,
                project_key = ?,
                updated_at = NULL,
                dirty = 0,
                conflict = 0,
                remote_snapshot = NULL,
                description = ?,
                reporter = ?,
                creator = ?,
                created_at = ?,
                resolution_date = ?,
                resolution = ?,
                labels = ?,
                fix_versions = ?,
                parent_key = ?,
                environment = ?,
                time_estimate = ?,
                time_spent = ?,
                time_remaining = ?,
                custom_fields = ?
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(&remote.summary)
        .bind(&remote.status)
        .bind(&remote.issue_type)
        .bind(&remote.priority)
        .bind(&remote.assignee)
        .bind(&remote.epic)
        .bind(remote.story_points)
        .bind(remote.sprint_id)
        .bind(&remote.project_key)
        .bind(&remote.description)
        .bind(&remote.reporter)
        .bind(&remote.creator)
        .bind(remote.created_at)
        .bind(remote.resolution_date)
        .bind(&remote.resolution)
        .bind(&remote.labels)
        .bind(&remote.fix_versions)
        .bind(&remote.parent_key)
        .bind(&remote.environment)
        .bind(&remote.time_estimate)
        .bind(&remote.time_spent)
        .bind(&remote.time_remaining)
        .bind(&remote.custom_fields)
        .bind(key)
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM issue_comments
            WHERE issue_key = ? AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;

        for comment in &remote.comments {
            sqlx::query(
                r#"
                INSERT INTO issue_comments (
                    id,
                    issue_key,
                    author,
                    body,
                    created_at,
                    updated_at,
                    profile_id,
                    dirty,
                    conflict,
                    remote_snapshot
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL)
                "#,
            )
            .bind(&comment.id)
            .bind(key)
            .bind(&comment.author)
            .bind(&comment.body)
            .bind(&comment.created_at)
            .bind(&comment.updated_at)
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
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
            INSERT INTO boards (id, name, type_name, location_json, updated_at, profile_id)
            VALUES (?, ?, ?, ?, strftime('%s','now'), ?)
            ON CONFLICT(id, profile_id) DO UPDATE SET
                name = excluded.name,
                type_name = excluded.type_name,
                location_json = excluded.location_json,
                updated_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(board_id as i64)
        .bind(name)
        .bind(type_name)
        .bind(location_json)
        .bind(self.profile_id())
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
            INSERT INTO board_config (board_id, estimation_type, estimation_field_id, raw_json, updated_at, profile_id)
            VALUES (?, ?, ?, NULL, strftime('%s','now'), ?)
            ON CONFLICT(board_id, profile_id) DO UPDATE SET
                estimation_type = excluded.estimation_type,
                estimation_field_id = excluded.estimation_field_id,
                updated_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(board_id as i64)
        .bind(estimation_type)
        .bind(estimation_field_id)
        .bind(self.profile_id())
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
        sqlx::query("DELETE FROM board_columns WHERE board_id = ? AND profile_id = ?")
            .bind(board_id as i64)
            .bind(self.profile_id())
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
                INSERT INTO board_columns (board_id, position, name, status_ids_json, profile_id)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(board_id as i64)
            .bind(idx as i64)
            .bind(&column.name)
            .bind(status_ids_json)
            .bind(self.profile_id())
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
            INSERT INTO sprints (id, board_id, name, state, start_date, end_date, complete_date, updated_at, profile_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), ?)
            ON CONFLICT(id, profile_id) DO UPDATE SET
                board_id = excluded.board_id,
                name = excluded.name,
                state = excluded.state,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                complete_date = excluded.complete_date,
                updated_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(sprint_id as i64)
        .bind(board_id as i64)
        .bind(name)
        .bind(state)
        .bind(start_date)
        .bind(end_date)
        .bind(complete_date)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn selected_board_ids(&self) -> Result<Vec<u64>> {
        let rows = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            WHERE profile_id = ?
            ORDER BY selected_at DESC
            "#,
        )
        .bind(self.profile_id())
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
            WHERE profile_id = ?
            ORDER BY name
            "#,
        )
        .bind(self.profile_id())
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
              AND profile_id = ?
            ORDER BY selected_at DESC
            LIMIT 1
            "#,
        )
        .bind(self.profile_id())
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
            WHERE profile_id = ?
            ORDER BY selected_at DESC
            LIMIT 1
            "#,
        )
        .bind(self.profile_id())
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
                  AND profile_id = ?
                "#,
            )
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO selected_boards (board_id, is_default, selected_at, profile_id)
            VALUES (?, ?, strftime('%s','now'), ?)
            ON CONFLICT(board_id, profile_id) DO UPDATE SET
                is_default = excluded.is_default,
                selected_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(board_id as i64)
        .bind(if is_default { 1 } else { 0 })
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn fetch_pending_outbox(&self, limit: i64) -> Result<Vec<OutboxRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, entity_type, entity_id, change_set, attempts
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
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                change_set: row.get("change_set"),
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

    pub async fn mark_outbox_conflict(&self, id: &str, error: Option<&str>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'conflict',
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

    pub async fn push_outbox(&self) -> Result<()> {
        let items = self.fetch_pending_outbox(OUTBOX_BATCH_SIZE).await?;
        for item in items {
            if item.attempts >= MAX_OUTBOX_ATTEMPTS {
                self.mark_outbox_failed(&item.id, "max attempts reached")
                    .await?;
                continue;
            }
            let conflict = match item.entity_type.as_str() {
                "issue" => self.issue_conflict(&item.entity_id).await?,
                "comment" => self.comment_conflict(&item.entity_id).await?,
                _ => false,
            };
            if conflict {
                self.mark_outbox_conflict(&item.id, Some("conflict"))
                    .await?;
                continue;
            }

            self.mark_outbox_processing(&item.id).await?;

            // TODO: push to Jira. For now we mark as done and clear dirty state.
            match item.entity_type.as_str() {
                "issue" => self.clear_issue_dirty(&item.entity_id).await?,
                "comment" => self.clear_comment_dirty(&item.entity_id).await?,
                _ => {}
            }
            self.mark_outbox_done(&item.id).await?;
        }
        Ok(())
    }

    pub async fn issue_conflict(&self, key: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT conflict
            FROM issues
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(|row| row.get::<i64, _>("conflict") != 0)
            .unwrap_or(false))
    }

    pub async fn comment_conflict(&self, id: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT conflict
            FROM issue_comments
            WHERE id = ? AND profile_id = ?
            "#,
        )
        .bind(id)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(|row| row.get::<i64, _>("conflict") != 0)
            .unwrap_or(false))
    }

    pub async fn clear_issue_dirty(&self, key: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET dirty = 0,
                conflict = 0,
                remote_snapshot = NULL
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn clear_comment_dirty(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issue_comments
            SET dirty = 0,
                conflict = 0,
                remote_snapshot = NULL
            WHERE id = ? AND profile_id = ?
            "#,
        )
        .bind(id)
        .bind(self.profile_id())
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
              AND profile_id = ?
            "#,
        )
        .bind(_board_id as i64)
        .bind(self.profile_id())
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
              AND profile_id = ?
            ORDER BY position
            "#,
        )
        .bind(_board_id as i64)
        .bind(self.profile_id())
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
            WHERE board_id = ? AND state = 'active' AND profile_id = ?
            ORDER BY start_date DESC
            LIMIT 1
            "#,
        )
        .bind(_board_id as i64)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        let Some(sprint) = sprint else {
            return Ok(Vec::new());
        };

        let rows = sqlx::query(
            r#"
            SELECT key, summary, status, issue_type, priority, assignee, epic, story_points,
                   sprint_id, project_key, updated_at, dirty, conflict, remote_snapshot,
                   description, reporter, creator, created_at, resolution_date, resolution,
                   labels, fix_versions, parent_key, environment,
                   time_estimate, time_spent, time_remaining, custom_fields
            FROM issues
            WHERE sprint_id = ? AND profile_id = ?
            "#,
        )
        .bind(sprint.get::<i64, _>("id"))
        .bind(self.profile_id())
        .fetch_all(&self.pool)
        .await?;

        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            let key: String = row.get("key");
            let project_key: Option<String> = row.get("project_key");
            let comments = self.fetch_comments_for_issue(&key).await?;
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
                comments,
                dirty: row.get::<i64, _>("dirty") != 0,
                conflict: row.get::<i64, _>("conflict") != 0,
                remote_snapshot: row.get("remote_snapshot"),

                // New fields
                description: row.get("description"),
                reporter: row.get("reporter"),
                creator: row.get("creator"),
                created_at: ts_to_system_time(row.get("created_at")),
                resolution_date: ts_to_system_time(row.get("resolution_date")),
                resolution: row.get("resolution"),
                labels: parse_json_array(row.get("labels")),
                fix_versions: parse_json_array(row.get("fix_versions")),
                parent_key: row.get("parent_key"),
                environment: row.get("environment"),
                time_estimate: row.get("time_estimate"),
                time_spent: row.get("time_spent"),
                time_remaining: row.get("time_remaining"),
                custom_fields: row.get("custom_fields"),
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
    async fn upsert_issues(&self, issues: &[IssueSummary]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        for issue in issues {
            let existing = sqlx::query(
                r#"
                SELECT dirty, conflict
                FROM issues
                WHERE key = ? AND profile_id = ?
                "#,
            )
            .bind(&issue.key)
            .bind(self.profile_id())
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(existing) = existing {
                let dirty: i64 = existing.get("dirty");
                let conflict: i64 = existing.get("conflict");
                if dirty != 0 || conflict != 0 {
                    continue;
                }
            }
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
                    raw_json,
                    profile_id,
                    dirty,
                    conflict,
                    remote_snapshot,
                    description,
                    reporter,
                    creator,
                    created_at,
                    resolution_date,
                    resolution,
                    labels,
                    fix_versions,
                    parent_key,
                    environment,
                    time_estimate,
                    time_spent,
                    time_remaining,
                    custom_fields
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(key, profile_id) DO UPDATE SET
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
                    synced_at = strftime('%s','now'),
                    profile_id = excluded.profile_id,
                    dirty = excluded.dirty,
                    conflict = excluded.conflict,
                    remote_snapshot = excluded.remote_snapshot,
                    description = excluded.description,
                    reporter = excluded.reporter,
                    creator = excluded.creator,
                    created_at = excluded.created_at,
                    resolution_date = excluded.resolution_date,
                    resolution = excluded.resolution,
                    labels = excluded.labels,
                    fix_versions = excluded.fix_versions,
                    parent_key = excluded.parent_key,
                    environment = excluded.environment,
                    time_estimate = excluded.time_estimate,
                    time_spent = excluded.time_spent,
                    time_remaining = excluded.time_remaining,
                    custom_fields = excluded.custom_fields
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
            .bind(self.profile_id())
            .bind(if issue.dirty { 1 } else { 0 })
            .bind(if issue.conflict { 1 } else { 0 })
            .bind(&issue.remote_snapshot)
            .bind(&issue.description)
            .bind(&issue.reporter)
            .bind(&issue.creator)
            .bind(system_time_to_ts(issue.created_at))
            .bind(system_time_to_ts(issue.resolution_date))
            .bind(&issue.resolution)
            .bind(serialize_json_array(&issue.labels))
            .bind(serialize_json_array(&issue.fix_versions))
            .bind(&issue.parent_key)
            .bind(&issue.environment)
            .bind(&issue.time_estimate)
            .bind(&issue.time_spent)
            .bind(&issue.time_remaining)
            .bind(&issue.custom_fields)
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
                    raw_json,
                    profile_id,
                    description,
                    reporter,
                    creator,
                    created_at,
                    resolution_date,
                    resolution,
                    labels,
                    fix_versions,
                    parent_key,
                    environment,
                    time_estimate,
                    time_spent,
                    time_remaining,
                    custom_fields
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            .bind(self.profile_id())
            .bind(&issue.description)
            .bind(&issue.reporter)
            .bind(&issue.creator)
            .bind(system_time_to_ts(issue.created_at))
            .bind(system_time_to_ts(issue.resolution_date))
            .bind(&issue.resolution)
            .bind(serialize_json_array(&issue.labels))
            .bind(serialize_json_array(&issue.fix_versions))
            .bind(&issue.parent_key)
            .bind(&issue.environment)
            .bind(&issue.time_estimate)
            .bind(&issue.time_spent)
            .bind(&issue.time_remaining)
            .bind(&issue.custom_fields)
            .execute(&mut *tx)
            .await?;

            let dirty_comment = sqlx::query(
                r#"
                SELECT 1
                FROM issue_comments
                WHERE issue_key = ?
                  AND profile_id = ?
                  AND (dirty = 1 OR conflict = 1)
                LIMIT 1
                "#,
            )
            .bind(&issue.key)
            .bind(self.profile_id())
            .fetch_optional(&mut *tx)
            .await?;
            if dirty_comment.is_some() {
                continue;
            }

            sqlx::query("DELETE FROM issue_comments WHERE issue_key = ? AND profile_id = ?")
                .bind(&issue.key)
                .bind(self.profile_id())
                .execute(&mut *tx)
                .await?;
            for comment in &issue.comments {
                if comment.id.is_empty() {
                    continue;
                }
                sqlx::query(
                    r#"
                    INSERT INTO issue_comments (
                        id,
                        issue_key,
                        author,
                        body,
                        created_at,
                        updated_at,
                        profile_id,
                        dirty,
                        conflict,
                        remote_snapshot
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id, profile_id) DO UPDATE SET
                        issue_key = excluded.issue_key,
                        author = excluded.author,
                        body = excluded.body,
                        created_at = excluded.created_at,
                        updated_at = excluded.updated_at,
                        profile_id = excluded.profile_id,
                        dirty = excluded.dirty,
                        conflict = excluded.conflict,
                        remote_snapshot = excluded.remote_snapshot
                    "#,
                )
                .bind(&comment.id)
                .bind(&issue.key)
                .bind(&comment.author)
                .bind(&comment.body)
                .bind(&comment.created_at)
                .bind(&comment.updated_at)
                .bind(self.profile_id())
                .bind(if comment.dirty { 1 } else { 0 })
                .bind(if comment.conflict { 1 } else { 0 })
                .bind(&comment.remote_snapshot)
                .execute(&mut *tx)
                .await?;
            }
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
        let entity_type = match command.entity_type {
            crate::data::model::OutboxEntityType::Issue => "issue",
            crate::data::model::OutboxEntityType::Comment => "comment",
        };
        let id = uuid::Uuid::now_v7().to_string();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO outbox (id, entity_type, entity_id, change_set, status, attempts, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'pending', 0, strftime('%s','now'), strftime('%s','now'))
            "#,
        )
        .bind(&id)
        .bind(entity_type)
        .bind(&command.entity_id)
        .bind(&command.change_set)
        .execute(&mut *tx)
        .await?;

        match command.entity_type {
            crate::data::model::OutboxEntityType::Issue => {
                sqlx::query(
                    r#"
                    UPDATE issues
                    SET dirty = 1
                    WHERE key = ? AND profile_id = ?
                    "#,
                )
                .bind(&command.entity_id)
                .bind(self.profile_id())
                .execute(&mut *tx)
                .await?;
            }
            crate::data::model::OutboxEntityType::Comment => {
                sqlx::query(
                    r#"
                    UPDATE issue_comments
                    SET dirty = 1
                    WHERE id = ? AND profile_id = ?
                    "#,
                )
                .bind(&command.entity_id)
                .bind(self.profile_id())
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
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
