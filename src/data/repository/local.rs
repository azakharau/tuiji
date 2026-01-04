use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};

use crate::{
    client::jira::BoardConfig,
    config::ProfileConfig,
    data::{
        BoardSummary, IssueSummary, SqliteRepository, SqliteRepositoryConfig, SyncState,
        repository::{AppRepository, QueryRepository, jira::JiraRepository},
    },
};

#[derive(Clone)]
pub struct RepositoryHub {
    cache: Arc<SqliteRepository>,
    remote: Option<Arc<JiraRepository>>,
}

impl RepositoryHub {
    pub async fn connect(
        cache_cfg: SqliteRepositoryConfig,
        profile: Option<&ProfileConfig>,
    ) -> Result<Self> {
        let cache = SqliteRepository::connect(cache_cfg).await?;
        let remote = match profile {
            Some(profile) => Some(Arc::new(JiraRepository::new(&profile.jira)?)),
            None => None,
        };
        Ok(Self {
            cache: Arc::new(cache),
            remote,
        })
    }

    pub fn with_profile(&self, profile: Option<&ProfileConfig>) -> Result<Self> {
        let remote = match profile {
            Some(profile) => Some(Arc::new(JiraRepository::new(&profile.jira)?)),
            None => None,
        };
        Ok(Self {
            cache: Arc::clone(&self.cache),
            remote,
        })
    }

    pub fn cache(&self) -> Arc<SqliteRepository> {
        Arc::clone(&self.cache)
    }
}

#[async_trait]
impl AppRepository for RepositoryHub {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig> {
        match &self.remote {
            Some(remote) => remote.board_config(board_id).await,
            None => Err(eyre!("Repository missing: no Jira profile configured")),
        }
    }

    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>> {
        match &self.remote {
            Some(remote) => remote.current_sprint_issues(board_id).await,
            None => Err(eyre!("Repository missing: no Jira profile configured")),
        }
    }

    async fn sync_state(&self) -> Result<SyncState> {
        self.cache.sync_state().await
    }

    async fn list_boards(&self) -> Result<Vec<BoardSummary>> {
        self.cache.list_boards().await
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
