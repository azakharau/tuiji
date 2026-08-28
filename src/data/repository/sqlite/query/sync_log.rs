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
        let direction = match filter {
            SyncLogFilter::All => None,
            SyncLogFilter::Pull => Some("pull"),
            SyncLogFilter::Push => Some("push"),
        };

        let rows = sqlx::query(
            r#"
            SELECT direction,
                   status,
                   error,
                   datetime(created_at, 'unixepoch') as created_at
            FROM sync_log
            WHERE profile_id = ?
              AND (? IS NULL OR direction = ?)
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(self.profile_id())
        .bind(direction)
        .bind(direction)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_direction_filter_should_select_only_that_direction() {
        let db_path =
            std::env::temp_dir().join(format!("tuiji-sync-log-{}.db", uuid::Uuid::now_v7()));
        let repo = SqliteRepository::connect(
            SqliteRepositoryConfig {
                db_path: db_path.clone(),
            },
            "test-profile".to_string(),
        )
        .await
        .unwrap();

        for direction in ["pull", "pull", "push"] {
            repo.log_sync_event_impl(direction, "ok", None, Some("test-profile"))
                .await
                .unwrap();
        }

        let count = async |filter| repo.sync_log_impl(10, filter).await.unwrap().len();

        assert_eq!(count(SyncLogFilter::All).await, 3);
        assert_eq!(count(SyncLogFilter::Pull).await, 2);
        assert_eq!(count(SyncLogFilter::Push).await, 1);

        let _ = std::fs::remove_file(db_path);
    }
}
