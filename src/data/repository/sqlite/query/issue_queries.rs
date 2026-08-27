use super::*;

const ISSUE_SELECT: &str = r#"
    SELECT key, summary, status, issue_type, priority, assignee, epic, story_points,
           sprint_id, project_key, updated_at, dirty, conflict, remote_snapshot,
           description, reporter, creator, created_at, resolution_date, resolution,
           labels, fix_versions, parent_key, environment,
           time_estimate, time_spent, time_remaining, custom_fields
    FROM issues
"#;

impl SqliteRepository {
    pub(crate) async fn fetch_comments_for_issue_impl(
        &self,
        issue_key: &str,
    ) -> Result<Vec<IssueComment>> {
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

    pub async fn issue_by_key(&self, key: &str) -> Result<Option<IssueSummary>> {
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(ISSUE_SELECT);
        query
            .push(" WHERE key = ")
            .push_bind(key)
            .push(" AND profile_id = ")
            .push_bind(self.profile_id());
        let row = query.build().fetch_optional(&self.pool).await?;

        match row {
            Some(row) => Ok(Some(self.issue_summary_from_row(row).await?)),
            None => Ok(None),
        }
    }

    pub(crate) async fn fetch_issue_impl(&self, issue_key: &str) -> Result<Option<IssueSummary>> {
        self.issue_by_key(issue_key).await
    }

    async fn issue_summary_from_row(&self, row: sqlx::sqlite::SqliteRow) -> Result<IssueSummary> {
        let key: String = row.get("key");
        let project_key: Option<String> = row.get("project_key");
        let comments = self.fetch_comments_for_issue_impl(&key).await?;
        Ok(IssueSummary {
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
        })
    }

    pub(crate) async fn current_sprint_issues_impl(
        &self,
        board_id: u64,
    ) -> Result<Vec<IssueSummary>> {
        let sprint = sqlx::query(
            r#"
            SELECT id
            FROM sprints
            WHERE board_id = ? AND state = 'active' AND profile_id = ?
            ORDER BY start_date DESC
            LIMIT 1
            "#,
        )
        .bind(board_id as i64)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        let Some(sprint) = sprint else {
            return Ok(Vec::new());
        };

        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(ISSUE_SELECT);
        query
            .push(" WHERE sprint_id = ")
            .push_bind(sprint.get::<i64, _>("id"))
            .push(" AND profile_id = ")
            .push_bind(self.profile_id());
        let rows = query.build().fetch_all(&self.pool).await?;

        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            issues.push(self.issue_summary_from_row(row).await?);
        }

        Ok(issues)
    }
}
