#![cfg(test)]

use super::*;

pub(super) struct TestFixture<'a> {
    board_id: u64,
    issue_keys: &'a [&'a str],
}

impl<'a> TestFixture<'a> {
    pub(super) fn new(board_id: u64, issue_keys: &'a [&'a str]) -> Self {
        Self {
            board_id,
            issue_keys,
        }
    }

    pub(super) async fn insert(self, repo: &SqliteRepository) -> Result<()> {
        let mut tx = repo.pool.begin().await?;
        let profile_id = repo.profile_id();
        let sprint_id = self.board_id as i64;

        sqlx::query("INSERT INTO boards (id, profile_id, name, type_name) VALUES (?, ?, ?, ?)")
            .bind(self.board_id as i64)
            .bind(profile_id)
            .bind("Test Board")
            .bind("scrum")
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO board_config \
             (board_id, profile_id, estimation_type, estimation_field_id) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(self.board_id as i64)
        .bind(profile_id)
        .bind("StoryPoints")
        .bind("customfield_10002")
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO board_columns \
             (board_id, profile_id, position, name, status_ids_json) \
             VALUES (?, ?, 0, 'TODO', '[]')",
        )
        .bind(self.board_id as i64)
        .bind(profile_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO sprints (id, profile_id, board_id, name, state, start_date) \
             VALUES (?, ?, ?, ?, 'active', ?)",
        )
        .bind(sprint_id)
        .bind(profile_id)
        .bind(self.board_id as i64)
        .bind("Test Sprint")
        .bind(current_ts())
        .execute(&mut *tx)
        .await?;

        for &key in self.issue_keys {
            sqlx::query(
                "INSERT INTO issues \
                 (key, profile_id, summary, status, issue_type, priority, assignee, \
                  sprint_id, project_key) \
                 VALUES (?, ?, ?, 'TODO', 'Task', 'Medium', 'Test User', ?, ?)",
            )
            .bind(key)
            .bind(profile_id)
            .bind(format!("Fixture {key}"))
            .bind(sprint_id)
            .bind(key.split('-').next())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::QueryRepository;

    #[tokio::test]
    async fn fixture_inserts_caller_selected_issues() {
        let db_path =
            std::env::temp_dir().join(format!("tuiji-fixture-{}.db", uuid::Uuid::now_v7()));
        let repo = SqliteRepository::connect(
            SqliteRepositoryConfig {
                db_path: db_path.clone(),
            },
            "test-profile".to_string(),
        )
        .await
        .unwrap();

        TestFixture::new(42, &["TEST-1", "TEST-2"])
            .insert(&repo)
            .await
            .unwrap();

        let mut keys = repo
            .current_sprint_issues(42)
            .await
            .unwrap()
            .into_iter()
            .map(|issue| issue.key)
            .collect::<Vec<_>>();
        keys.sort();
        assert_eq!(keys, vec!["TEST-1".to_string(), "TEST-2".to_string()]);

        let _ = std::fs::remove_file(db_path);
    }
}
