use super::*;

impl SqliteRepository {
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
