use super::*;

impl SqliteRepository {
    pub async fn fetch_pending_outbox(&self, limit: i64) -> Result<Vec<OutboxRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, entity_type, entity_id, change_set, attempts
            FROM outbox
            WHERE status = 'pending'
              AND profile_id = ?
            ORDER BY created_at
            LIMIT ?
            "#,
        )
        .bind(self.profile_id())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(OutboxRecord {
                id: row.get("id"),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                change_set: row.get("change_set"),
                attempts: row.get("attempts"),
            });
        }
        Ok(items)
    }

    pub async fn mark_outbox_processing(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'processing',
                attempts = attempts + 1,
                updated_at = strftime('%s','now')
            WHERE id = ?
              AND profile_id = ?
            "#,
        )
        .bind(id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_outbox_done(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'done',
                updated_at = strftime('%s','now')
            WHERE id = ?
              AND profile_id = ?
            "#,
        )
        .bind(id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_outbox_failed(&self, id: &str, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'failed',
                last_error = ?,
                updated_at = strftime('%s','now')
            WHERE id = ?
              AND profile_id = ?
            "#,
        )
        .bind(error)
        .bind(id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_outbox_conflict(&self, id: &str, error: Option<&str>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'conflict',
                last_error = ?,
                updated_at = strftime('%s','now')
            WHERE id = ?
              AND profile_id = ?
            "#,
        )
        .bind(error)
        .bind(id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn outbox_entity_conflict(&self, entity_type: &str, entity_id: &str) -> Result<bool> {
        match entity_type {
            "issue" => self.issue_conflict(entity_id).await,
            "comment" => self.comment_conflict(entity_id).await,
            _ => Ok(false),
        }
    }

    pub async fn clear_outbox_entity_dirty(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<()> {
        match entity_type {
            "issue" => self.clear_issue_dirty(entity_id).await,
            "comment" => self.clear_comment_dirty(entity_id).await,
            _ => Ok(()),
        }
    }

    pub(super) async fn enqueue_outbox_impl(&self, command: OutboxCommand) -> Result<()> {
        let entity_type = match command.entity_type {
            crate::data::model::OutboxEntityType::Issue => "issue",
            crate::data::model::OutboxEntityType::Comment => "comment",
        };
        let id = uuid::Uuid::now_v7().to_string();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO outbox (id, profile_id, entity_type, entity_id, change_set, status, attempts, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 'pending', 0, strftime('%s','now'), strftime('%s','now'))
            "#,
        )
        .bind(&id)
        .bind(self.profile_id())
        .bind(entity_type)
        .bind(&command.entity_id)
        .bind(&command.change_set)
        .execute(&mut *tx)
        .await?;

        match command.entity_type {
            crate::data::model::OutboxEntityType::Issue => {
                sqlx::query(
                    r#"
                    UPDATE issues
                    SET dirty = 1
                    WHERE key = ? AND profile_id = ?
                    "#,
                )
                .bind(&command.entity_id)
                .bind(self.profile_id())
                .execute(&mut *tx)
                .await?;
            }
            crate::data::model::OutboxEntityType::Comment => {
                sqlx::query(
                    r#"
                    UPDATE issue_comments
                    SET dirty = 1
                    WHERE id = ? AND profile_id = ?
                    "#,
                )
                .bind(&command.entity_id)
                .bind(self.profile_id())
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }
}
