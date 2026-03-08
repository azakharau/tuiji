use super::*;

impl SqliteRepository {
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
}
