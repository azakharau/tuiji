use super::*;

impl SqliteRepository {
    pub async fn cache_transitions(
        &self,
        key: &str,
        status: &str,
        choices: &[TransitionChoice],
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO issue_transitions
                (issue_key, profile_id, status, choices_json, fetched_at)
            VALUES (?, ?, ?, ?, strftime('%s','now'))
            ON CONFLICT (issue_key, profile_id) DO UPDATE SET
                status = excluded.status,
                choices_json = excluded.choices_json,
                fetched_at = excluded.fetched_at
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .bind(status)
        .bind(serde_json::to_string(choices)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Returns the cached choices together with the issue status they were
    /// fetched under, so the caller can reject a list that no longer applies.
    pub async fn cached_transitions(
        &self,
        key: &str,
    ) -> Result<Option<(String, Vec<TransitionChoice>)>> {
        let row = sqlx::query(
            r#"
            SELECT status, choices_json
            FROM issue_transitions
            WHERE issue_key = ?
              AND profile_id = ?
            "#,
        )
        .bind(key)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let status: String = row.try_get("status")?;
        let choices_json: String = row.try_get("choices_json")?;
        Ok(Some((status, serde_json::from_str(&choices_json)?)))
    }
}
