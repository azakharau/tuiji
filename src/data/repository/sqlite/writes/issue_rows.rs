use super::*;

mod history_row;
mod issue_row;
mod skip_check;

impl SqliteRepository {
    pub async fn apply_issue_patch(
        &self,
        key: &str,
        patch: &crate::data::model::IssuePatch,
    ) -> Result<()> {
        if patch.is_empty() {
            return Ok(());
        }

        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
            "UPDATE issues SET updated_at = strftime('%s','now'), dirty = 1",
        );
        if let Some(summary) = &patch.summary {
            query.push(", summary = ").push_bind(summary);
        }
        if let Some(description) = &patch.description {
            query.push(", description = ").push_bind(description);
        }
        if let Some(priority) = &patch.priority {
            query.push(", priority = ").push_bind(priority);
        }
        query
            .push(" WHERE key = ")
            .push_bind(key)
            .push(" AND profile_id = ")
            .push_bind(self.profile_id());
        query.build().execute(&self.pool).await?;

        Ok(())
    }

    pub async fn set_issue_status(&self, key: &str, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET status = ?,
                updated_at = strftime('%s','now'),
                dirty = 1
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(status)
        .bind(key)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_issue_assignee(&self, key: &str, display_name: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET assignee = ?,
                updated_at = strftime('%s','now'),
                dirty = 1
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(display_name)
        .bind(key)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
