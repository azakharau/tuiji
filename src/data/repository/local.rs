use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::Result;

use crate::{
    config::{ProfileConfig, SyncMode},
    data::{
        BoardConfig, BoardSummary, IssueSummary, SqliteRepository, SqliteRepositoryConfig,
        SyncLogEntry, SyncLogFilter, SyncState,
        repository::{
            BoardRepository, ConflictRepository, IssueRepository, QueryRepository,
            SyncStatusRepository, jira::JiraRepository,
        },
    },
};

mod conflicts;
mod online;
mod sync;

const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Clone)]
pub struct RepositoryHub {
    cache: Arc<SqliteRepository>,
    remote: Option<Arc<JiraRepository>>,
    sync_mode: SyncMode,
    profile_id: String,
}

impl RepositoryHub {
    pub async fn connect(
        cache_cfg: SqliteRepositoryConfig,
        profile: Option<&ProfileConfig>,
    ) -> Result<Self> {
        let profile_id = resolve_profile_id(profile);
        let cache = SqliteRepository::connect(cache_cfg, profile_id.clone()).await?;
        let sync_mode = resolve_sync_mode(profile);
        let remote = build_remote(profile, sync_mode)?;
        Ok(Self {
            cache: Arc::new(cache),
            remote,
            sync_mode,
            profile_id,
        })
    }

    pub fn with_profile(&self, profile: Option<&ProfileConfig>) -> Result<Self> {
        let sync_mode = resolve_sync_mode(profile);
        let profile_id = resolve_profile_id(profile);
        let remote = build_remote(profile, sync_mode)?;
        Ok(Self {
            cache: Arc::new(self.cache.with_profile(profile_id.clone())),
            remote,
            sync_mode,
            profile_id,
        })
    }

    pub fn cache(&self) -> Arc<SqliteRepository> {
        Arc::clone(&self.cache)
    }

    pub fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }
}

#[async_trait]
impl BoardRepository for RepositoryHub {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig> {
        self.board_config_impl(board_id).await
    }

    async fn list_boards(&self) -> Result<Vec<BoardSummary>> {
        self.list_boards_impl().await
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

#[async_trait]
impl IssueRepository for RepositoryHub {
    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>> {
        self.current_sprint_issues_impl(board_id).await
    }
}

#[async_trait]
impl ConflictRepository for RepositoryHub {
    async fn conflict_issues(&self) -> Result<Vec<IssueSummary>> {
        self.cache.list_conflict_issues().await
    }

    async fn conflict_count(&self) -> Result<usize> {
        self.cache.count_conflicts().await
    }

    async fn resolve_conflict_use_local(&self, key: &str) -> Result<()> {
        self.cache.resolve_issue_use_local(key).await
    }

    async fn resolve_conflict_use_remote(&self, key: &str) -> Result<()> {
        self.resolve_conflict_use_remote_impl(key).await
    }
}

#[async_trait]
impl SyncStatusRepository for RepositoryHub {
    async fn sync_state(&self) -> Result<SyncState> {
        self.cache.sync_state().await
    }

    async fn sync_log(&self, limit: usize, filter: SyncLogFilter) -> Result<Vec<SyncLogEntry>> {
        self.cache.sync_log(limit, filter).await
    }
}

fn resolve_profile_id(profile: Option<&ProfileConfig>) -> String {
    profile
        .map(|profile| profile.id.clone())
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string())
}

fn resolve_sync_mode(profile: Option<&ProfileConfig>) -> SyncMode {
    profile
        .map(|profile| profile.sync_mode())
        .unwrap_or(SyncMode::Cache)
}

fn build_remote(
    profile: Option<&ProfileConfig>,
    sync_mode: SyncMode,
) -> Result<Option<Arc<JiraRepository>>> {
    match profile {
        Some(profile) if sync_mode == SyncMode::Online => {
            Ok(Some(Arc::new(JiraRepository::new(&profile.jira)?)))
        }
        _ => Ok(None),
    }
}
