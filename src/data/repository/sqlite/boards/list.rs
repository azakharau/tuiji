use super::*;

impl SqliteRepository {
    pub async fn list_boards(&self) -> Result<Vec<BoardSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, type_name
            FROM boards
            WHERE profile_id = ?
            ORDER BY name
            "#,
        )
        .bind(self.profile_id())
        .fetch_all(&self.pool)
        .await?;

        let mut boards = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            boards.push(BoardSummary {
                id: id as u64,
                name: row.get("name"),
                type_name: row.get("type_name"),
            });
        }
        Ok(boards)
    }
}
