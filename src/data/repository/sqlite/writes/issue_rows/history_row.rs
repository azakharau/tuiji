use super::*;

impl SqliteRepository {
    pub(crate) async fn insert_issue_history_row(
        &self,
        tx: &mut SqliteTx<'_>,
        issue: &IssueSummary,
        snapshot_at: i64,
    ) -> Result<()> {
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
