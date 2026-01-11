use async_trait::async_trait;
use color_eyre::Result;

use crate::client::jira::BoardConfig;
use crate::data::model::{IssueSummary, OutboxCommand, SyncLogEntry, SyncLogFilter, SyncState};

pub mod jira;
pub mod local;
pub mod sqlite;

#[async_trait]
pub trait QueryRepository: Send + Sync {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig>;
    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>>;
    async fn sync_state(&self) -> Result<SyncState>;
}

#[async_trait]
pub trait CommandRepository: Send + Sync {
    async fn upsert_issues(&self, _issues: &[IssueSummary]) -> Result<()> {
        Ok(())
    }
    async fn set_sync_state(&self, _state: SyncState) -> Result<()> {
        Ok(())
    }
    async fn enqueue_outbox(&self, _command: OutboxCommand) -> Result<()> {
        Ok(())
    }
}

pub trait Repository: QueryRepository + CommandRepository {}

impl<T> Repository for T where T: QueryRepository + CommandRepository {}

#[async_trait]
pub trait AppRepository: Send + Sync {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig>;
    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>>;
    async fn sync_state(&self) -> Result<SyncState>;
    async fn list_boards(&self) -> Result<Vec<crate::data::BoardSummary>>;
    async fn default_board_id(&self) -> Result<Option<u64>>;
    async fn set_selected_board(&self, board_id: u64, is_default: bool) -> Result<()>;
    async fn selected_board_ids(&self) -> Result<Vec<u64>>;
    async fn seed_mock_data_if_empty(&self) -> Result<Option<u64>>;
    async fn conflict_issues(&self) -> Result<Vec<IssueSummary>>;
    async fn conflict_count(&self) -> Result<usize>;
    async fn resolve_conflict_use_local(&self, key: &str) -> Result<()>;
    async fn resolve_conflict_use_remote(&self, key: &str) -> Result<()>;
    async fn sync_log(&self, limit: usize, filter: SyncLogFilter) -> Result<Vec<SyncLogEntry>>;
}
