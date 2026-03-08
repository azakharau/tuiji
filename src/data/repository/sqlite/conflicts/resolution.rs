use super::*;

impl SqliteRepository {
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
}
