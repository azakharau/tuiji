use super::*;

impl SqliteRepository {
    pub(crate) async fn should_skip_issue(
        &self,
        tx: &mut SqliteTx<'_>,
        issue_key: &str,
    ) -> Result<bool> {
        let existing = sqlx::query(
            r#"
            SELECT dirty, conflict
            FROM issues
            WHERE key = ? AND profile_id = ?
            "#,
        )
        .bind(issue_key)
        .bind(self.profile_id())
        .fetch_optional(&mut **tx)
        .await?;

        let Some(existing) = existing else {
            return Ok(false);
        };

        let dirty: i64 = existing.get("dirty");
        let conflict: i64 = existing.get("conflict");
        Ok(dirty != 0 || conflict != 0)
    }
}
