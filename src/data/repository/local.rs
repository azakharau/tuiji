use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use gouqi::Board;

use crate::{
    client::jira::BoardConfig,
    config::{ProfileConfig, SyncMode},
    data::{
        BoardSummary, IssueSummary, SqliteRepository, SqliteRepositoryConfig, SyncState,
        repository::{AppRepository, CommandRepository, QueryRepository, jira::JiraRepository},
    },
};

#[derive(Clone)]
pub struct RepositoryHub {
    cache: Arc<SqliteRepository>,
    remote: Option<Arc<JiraRepository>>,
    sync_mode: SyncMode,
}

impl RepositoryHub {
    pub async fn connect(
        cache_cfg: SqliteRepositoryConfig,
        profile: Option<&ProfileConfig>,
    ) -> Result<Self> {
        let cache = SqliteRepository::connect(cache_cfg).await?;
        let sync_mode = profile
            .map(|profile| profile.sync_mode())
            .unwrap_or(SyncMode::Cache);
        let remote = match profile {
            Some(profile) if sync_mode == SyncMode::Online => {
                Some(Arc::new(JiraRepository::new(&profile.jira)?))
            }
            None => None,
            _ => None,
        };
        Ok(Self {
            cache: Arc::new(cache),
            remote,
            sync_mode,
        })
    }

    pub fn with_profile(&self, profile: Option<&ProfileConfig>) -> Result<Self> {
        let sync_mode = profile
            .map(|profile| profile.sync_mode())
            .unwrap_or(SyncMode::Cache);
        let remote = match profile {
            Some(profile) if sync_mode == SyncMode::Online => {
                Some(Arc::new(JiraRepository::new(&profile.jira)?))
            }
            None => None,
            _ => None,
        };
        Ok(Self {
            cache: Arc::clone(&self.cache),
            remote,
            sync_mode,
        })
    }

    pub fn cache(&self) -> Arc<SqliteRepository> {
        Arc::clone(&self.cache)
    }

    pub fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }

    pub async fn sync_pull(&self) -> Result<()> {
        let Some(remote) = &self.remote else {
            self.cache
                .log_sync_event("pull", "error", Some("sync pull requires online mode"))
                .await?;
            return Err(eyre!("Sync pull requires online mode"));
        };
        let board_ids = self.cache.selected_board_ids().await?;
        let result: Result<()> = async {
            for board_id in board_ids {
                let config = remote.board_config(board_id).await?;
                self.cache.upsert_board_config(board_id, &config).await?;
                self.cache
                    .replace_board_columns(board_id, &config.columns)
                    .await?;
                let issues = remote.current_sprint_issues(board_id).await?;
                self.cache.upsert_issues(issues).await?;
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.cache.log_sync_event("pull", "success", None).await?;
                Ok(())
            }
            Err(err) => {
                self.cache
                    .log_sync_event("pull", "error", Some(&err.to_string()))
                    .await?;
                Err(err)
            }
        }
    }

    pub async fn sync_push(&self) -> Result<()> {
        let Some(_remote) = &self.remote else {
            self.cache
                .log_sync_event("push", "error", Some("sync push requires online mode"))
                .await?;
            return Err(eyre!("Sync push requires online mode"));
        };
        let result = self.cache.push_outbox().await;
        match result {
            Ok(()) => {
                self.cache.log_sync_event("push", "success", None).await?;
                Ok(())
            }
            Err(err) => {
                self.cache
                    .log_sync_event("push", "error", Some(&err.to_string()))
                    .await?;
                Err(err)
            }
        }
    }
}

#[async_trait]
impl AppRepository for RepositoryHub {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig> {
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

    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>> {
        match self.sync_mode {
            SyncMode::Online => match &self.remote {
                Some(remote) => match remote.current_sprint_issues(board_id).await {
                    Ok(issues) => {
                        self.cache.upsert_issues(issues.clone()).await?;
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

    async fn sync_state(&self) -> Result<SyncState> {
        self.cache.sync_state().await
    }

    async fn list_boards(&self) -> Result<Vec<BoardSummary>> {
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

    async fn default_board_id(&self) -> Result<Option<u64>> {
        self.cache.default_board_id().await
    }

    async fn set_selected_board(&self, board_id: u64, is_default: bool) -> Result<()> {
        self.cache.set_selected_board(board_id, is_default).await
    }

    async fn selected_board_ids(&self) -> Result<Vec<u64>> {
        self.cache.selected_board_ids().await
    }

    async fn seed_mock_data_if_empty(&self) -> Result<Option<u64>> {
        self.cache.seed_mock_data_if_empty().await
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
