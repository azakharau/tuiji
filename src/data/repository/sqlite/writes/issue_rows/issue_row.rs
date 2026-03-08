use super::*;

impl SqliteRepository {
    pub(crate) async fn upsert_issue_row(
        &self,
        tx: &mut SqliteTx<'_>,
        issue: &IssueSummary,
        updated_at: Option<i64>,
    ) -> Result<()> {
        let project_key = issue
            .project_key
            .clone()
            .or_else(|| issue.key.split('-').next().map(ToString::to_string));

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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
