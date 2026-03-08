use color_eyre::{Result, eyre::eyre};
use gouqi::Board;

use super::*;
use crate::data::repository::{CommandRepository, QueryRepository};

impl RepositoryHub {
    pub(super) async fn board_config_impl(&self, board_id: u64) -> Result<BoardConfig> {
        match self.sync_mode {
            SyncMode::Online => match &self.remote {
                Some(remote) => match remote.board_config(board_id).await {
                    Ok(config) => {
                        self.cache.upsert_board_config(board_id, &config).await?;
                        self.cache
                            .replace_board_columns(board_id, &config.columns)
                            .await?;
                        Ok(config)
                    }
                    Err(err) => self.cache.board_config(board_id).await.or(Err(err)),
                },
                None => Err(eyre!("Repository missing: no Jira profile configured")),
            },
            SyncMode::Cache => self.cache.board_config(board_id).await,
        }
    }

    pub(super) async fn current_sprint_issues_impl(
        &self,
        board_id: u64,
    ) -> Result<Vec<IssueSummary>> {
        match self.sync_mode {
            SyncMode::Online => match &self.remote {
                Some(remote) => match remote.current_sprint_issues(board_id).await {
                    Ok(issues) => {
                        self.cache.upsert_issues(&issues).await?;
                        Ok(issues)
                    }
                    Err(err) => self
                        .cache
                        .current_sprint_issues(board_id)
                        .await
                        .or(Err(err)),
                },
                None => Err(eyre!("Repository missing: no Jira profile configured")),
            },
            SyncMode::Cache => self.cache.current_sprint_issues(board_id).await,
        }
    }

    pub(super) async fn list_boards_impl(&self) -> Result<Vec<BoardSummary>> {
        match self.sync_mode {
            SyncMode::Online => match &self.remote {
                Some(remote) => {
                    let boards = remote.list_boards().await?;
                    for board in &boards {
                        self.cache
                            .upsert_board(
                                board.id,
                                board.name.as_str(),
                                board.type_name.as_str(),
                                board_location_json(board),
                            )
                            .await?;
                    }
                    Ok(boards
                        .into_iter()
                        .map(|board| BoardSummary {
                            id: board.id,
                            name: board.name,
                            type_name: Some(board.type_name),
                        })
                        .collect())
                }
                None => Err(eyre!("Repository missing: no Jira profile configured")),
            },
            SyncMode::Cache => self.cache.list_boards().await,
        }
    }
}

fn board_location_json(board: &Board) -> Option<String> {
    board.location.as_ref().map(|loc| {
        serde_json::json!({
            "project_id": loc.project_id,
            "user_id": loc.user_id,
            "user_account_id": loc.user_account_id,
            "display_name": loc.display_name,
            "project_name": loc.project_name,
            "project_key": loc.project_key,
            "project_type_key": loc.project_type_key,
            "name": loc.name,
        })
        .to_string()
    })
}
