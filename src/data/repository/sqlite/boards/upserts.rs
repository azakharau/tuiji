use super::*;

impl SqliteRepository {
    pub async fn upsert_board(
        &self,
        board_id: u64,
        name: &str,
        type_name: &str,
        location_json: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO boards (id, name, type_name, location_json, updated_at, profile_id)
            VALUES (?, ?, ?, ?, strftime('%s','now'), ?)
            ON CONFLICT(id, profile_id) DO UPDATE SET
                name = excluded.name,
                type_name = excluded.type_name,
                location_json = excluded.location_json,
                updated_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(board_id as i64)
        .bind(name)
        .bind(type_name)
        .bind(location_json)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_board_config(&self, board_id: u64, config: &BoardConfig) -> Result<()> {
        let (estimation_type, estimation_field_id) = match &config.estimation {
            Estimation::StoryPoints(field_id) => ("StoryPoints", field_id.as_str()),
            Estimation::DateBased(field_id) => ("DateBased", field_id.as_str()),
        };
        sqlx::query(
            r#"
            INSERT INTO board_config (board_id, estimation_type, estimation_field_id, raw_json, updated_at, profile_id)
            VALUES (?, ?, ?, NULL, strftime('%s','now'), ?)
            ON CONFLICT(board_id, profile_id) DO UPDATE SET
                estimation_type = excluded.estimation_type,
                estimation_field_id = excluded.estimation_field_id,
                updated_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(board_id as i64)
        .bind(estimation_type)
        .bind(estimation_field_id)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn replace_board_columns(
        &self,
        board_id: u64,
        columns: &[BoardColumn],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM board_columns WHERE board_id = ? AND profile_id = ?")
            .bind(board_id as i64)
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        for (idx, column) in columns.iter().enumerate() {
            let status_ids = column
                .statuses
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<String>>();
            let status_ids_json = serde_json::to_string(&status_ids)?;
            sqlx::query(
                r#"
                INSERT INTO board_columns (board_id, position, name, status_ids_json, profile_id)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(board_id as i64)
            .bind(idx as i64)
            .bind(&column.name)
            .bind(status_ids_json)
            .bind(self.profile_id())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_sprint(&self, sprint: SprintUpsert<'_>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sprints (id, board_id, name, state, start_date, end_date, complete_date, updated_at, profile_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), ?)
            ON CONFLICT(id, profile_id) DO UPDATE SET
                board_id = excluded.board_id,
                name = excluded.name,
                state = excluded.state,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                complete_date = excluded.complete_date,
                updated_at = strftime('%s','now'),
                profile_id = excluded.profile_id
            "#,
        )
        .bind(sprint.sprint_id as i64)
        .bind(sprint.board_id as i64)
        .bind(sprint.name)
        .bind(sprint.state)
        .bind(sprint.start_date)
        .bind(sprint.end_date)
        .bind(sprint.complete_date)
        .bind(self.profile_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
