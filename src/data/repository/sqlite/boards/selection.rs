use super::*;

impl SqliteRepository {
    pub async fn selected_board_ids(&self) -> Result<Vec<u64>> {
        let rows = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            WHERE profile_id = ?
            ORDER BY selected_at DESC
            "#,
        )
        .bind(self.profile_id())
        .fetch_all(&self.pool)
        .await?;

        let mut ids = Vec::with_capacity(rows.len());
        for row in rows {
            let board_id: i64 = row.get("board_id");
            ids.push(board_id as u64);
        }
        Ok(ids)
    }

    pub async fn default_board_id(&self) -> Result<Option<u64>> {
        let row = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            WHERE is_default = 1
              AND profile_id = ?
            ORDER BY selected_at DESC
            LIMIT 1
            "#,
        )
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let board_id: i64 = row.get("board_id");
            return Ok(Some(board_id as u64));
        }

        let fallback = sqlx::query(
            r#"
            SELECT board_id
            FROM selected_boards
            WHERE profile_id = ?
            ORDER BY selected_at DESC
            LIMIT 1
            "#,
        )
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        Ok(fallback.map(|row| row.get::<i64, _>("board_id") as u64))
    }

    pub async fn set_selected_board(&self, board_id: u64, is_default: bool) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        if is_default {
            sqlx::query(
                r#"
                UPDATE selected_boards
                SET is_default = 0
                WHERE is_default = 1
                  AND profile_id = ?
                "#,
            )
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO selected_boards (board_id, is_default, selected_at, profile_id)
            VALUES (?, ?, strftime('%s','now'), ?)
            ON CONFLICT(board_id, profile_id) DO UPDATE SET
                is_default = excluded.is_default,
                selected_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(board_id as i64)
        .bind(if is_default { 1 } else { 0 })
        .bind(self.profile_id())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
