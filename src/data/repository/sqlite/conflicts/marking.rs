use super::*;

impl SqliteRepository {
    pub async fn mark_issue_conflict(&self, key: &str, remote_snapshot: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET remote_snapshot = ?
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(remote_snapshot)
        .bind(key)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        self.set_issue_conflict(key).await
    }

    pub async fn set_issue_conflict(&self, key: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE issues
            SET conflict = 1
            WHERE key = ? AND profile_id = ?
            "#,
        )
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
}
