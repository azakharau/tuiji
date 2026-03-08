use super::*;

impl SqliteRepository {
    pub(crate) async fn persist_sync_state(&self, state: SyncState) -> Result<()> {
        let last_full_sync = system_time_to_ts(state.last_full_sync);
        let last_pull = system_time_to_ts(state.last_pull);
        let last_push = system_time_to_ts(state.last_push);
        sqlx::query(
            r#"
            INSERT INTO sync_state (id, last_full_sync, last_pull, last_push, updated_at)
            VALUES (1, ?, ?, ?, strftime('%s','now'))
            ON CONFLICT(id) DO UPDATE SET
                last_full_sync = excluded.last_full_sync,
                last_pull = excluded.last_pull,
                last_push = excluded.last_push,
                updated_at = strftime('%s','now')
            "#,
        )
        .bind(last_full_sync)
        .bind(last_pull)
        .bind(last_push)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
