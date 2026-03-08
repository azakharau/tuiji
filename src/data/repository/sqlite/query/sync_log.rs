use super::*;

impl SqliteRepository {
    pub(crate) async fn log_sync_event_impl(
        &self,
        direction: &str,
        status: &str,
        error: Option<&str>,
        profile_id: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sync_log (direction, status, error, profile_id, created_at)
            VALUES (?, ?, ?, ?, strftime('%s','now'))
            "#,
        )
        .bind(direction)
        .bind(status)
        .bind(error)
        .bind(profile_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn sync_log_impl(
        &self,
        limit: usize,
        filter: SyncLogFilter,
    ) -> Result<Vec<SyncLogEntry>> {
        let mut sql = String::from(
            r#"
            SELECT direction,
                   status,
                   error,
                   datetime(created_at, 'unixepoch') as created_at
            FROM sync_log
            WHERE profile_id = ?
            "#,
        );
        if !matches!(filter, SyncLogFilter::All) {
            sql.push_str(" AND direction = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?");

        let mut query = sqlx::query(&sql).bind(self.profile_id());
        if !matches!(filter, SyncLogFilter::All) {
            let direction = match filter {
                SyncLogFilter::Pull => "pull",
                SyncLogFilter::Push => "push",
                SyncLogFilter::All => "pull",
            };
            query = query.bind(direction);
        }
        query = query.bind(limit as i64);

        let rows = query.fetch_all(&self.pool).await?;
        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let direction: String = row.get("direction");
            let status: String = row.get("status");
            let error: Option<String> = row.get("error");
            let created_at: String = row.get("created_at");
            entries.push(SyncLogEntry {
                direction,
                status,
                error,
                created_at,
            });
        }
        Ok(entries)
    }

    pub(crate) async fn load_sync_state(&self) -> Result<SyncState> {
        let row = sqlx::query(
            r#"
            SELECT last_full_sync, last_pull, last_push
            FROM sync_state
            WHERE id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(SyncState::default());
        };

        Ok(SyncState {
            last_full_sync: ts_to_system_time(row.get("last_full_sync")),
            last_pull: ts_to_system_time(row.get("last_pull")),
            last_push: ts_to_system_time(row.get("last_push")),
        })
    }
}
