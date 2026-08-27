use super::*;

impl SqliteRepository {
    pub async fn insert_local_comment(&self, issue_key: &str, id: &str, body: &str) -> Result<()> {
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
                dirty
            )
            VALUES (?, ?, '', ?, strftime('%s','now'), strftime('%s','now'), ?, 1)
            "#,
        )
        .bind(id)
        .bind(issue_key)
        .bind(body)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_comment(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM issue_comments WHERE id = ? AND profile_id = ?")
            .bind(id)
            .bind(self.profile_id())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub(super) async fn has_dirty_comments(
        &self,
        tx: &mut SqliteTx<'_>,
        issue_key: &str,
    ) -> Result<bool> {
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
        .bind(issue_key)
        .bind(self.profile_id())
        .fetch_optional(&mut **tx)
        .await?;

        Ok(dirty_comment.is_some())
    }

    pub(super) async fn replace_issue_comments(
        &self,
        tx: &mut SqliteTx<'_>,
        issue: &IssueSummary,
    ) -> Result<()> {
        sqlx::query("DELETE FROM issue_comments WHERE issue_key = ? AND profile_id = ?")
            .bind(&issue.key)
            .bind(self.profile_id())
            .execute(&mut **tx)
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
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}
