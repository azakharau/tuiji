use super::*;

impl SqliteRepository {
    pub(crate) async fn board_config_impl(&self, board_id: u64) -> Result<BoardConfig> {
        let record = sqlx::query(
            r#"
            SELECT estimation_type, estimation_field_id
            FROM board_config
            WHERE board_id = ?
              AND profile_id = ?
            "#,
        )
        .bind(board_id as i64)
        .bind(self.profile_id())
        .fetch_optional(&self.pool)
        .await?;

        let record =
            record.ok_or_else(|| eyre!("Board config not found for board {}", board_id))?;
        let estimation_type: String = record.get("estimation_type");
        let estimation_field_id: String = record.get("estimation_field_id");
        let estimation = match estimation_type.as_str() {
            "StoryPoints" | "storypoints" | "story_points" | "story" => {
                Estimation::StoryPoints(estimation_field_id)
            }
            "DateBased" | "datebased" | "days" | "date" => {
                Estimation::DateBased(estimation_field_id)
            }
            _ => Estimation::StoryPoints(estimation_field_id),
        };

        let column_rows = sqlx::query(
            r#"
            SELECT name, status_ids_json
            FROM board_columns
            WHERE board_id = ?
              AND profile_id = ?
            ORDER BY position
            "#,
        )
        .bind(board_id as i64)
        .bind(self.profile_id())
        .fetch_all(&self.pool)
        .await?;

        let mut columns = Vec::with_capacity(column_rows.len());
        for row in column_rows {
            let name: String = row.get("name");
            let status_ids_json: String = row.get("status_ids_json");
            let status_ids: Vec<String> =
                serde_json::from_str(&status_ids_json).unwrap_or_default();
            let statuses = status_ids
                .into_iter()
                .map(|id| ColumnStatusRef {
                    id,
                    self_link: String::new(),
                })
                .collect();
            let mut column = BoardColumn { name, statuses };
            column.name = column.name.to_uppercase();
            columns.push(column);
        }

        Ok(BoardConfig {
            columns,
            estimation,
        })
    }
}
