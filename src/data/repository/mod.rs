use async_trait::async_trait;
use color_eyre::Result;

use crate::data::model::{
    BoardConfig, IssueMutation, IssueSummary, OutboxCommand, SyncLogEntry, SyncLogFilter,
    SyncState, TransitionChoice,
};

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
pub trait BoardRepository: Send + Sync {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig>;
    async fn list_boards(&self) -> Result<Vec<crate::data::BoardSummary>>;
    async fn default_board_id(&self) -> Result<Option<u64>>;
    async fn set_selected_board(&self, board_id: u64, is_default: bool) -> Result<()>;
    async fn selected_board_ids(&self) -> Result<Vec<u64>>;
}

/// Writes. Mutations that target an existing issue are applied to the cache and
/// enqueued in the outbox, so they survive being offline. `IssueMutation::Create`
/// and the metadata lookups require a live connection.
#[async_trait]
pub trait MutationRepository: Send + Sync {
    async fn apply_mutation(&self, mutation: IssueMutation) -> Result<()>;
    async fn available_transitions(&self, key: &str) -> Result<Vec<TransitionChoice>>;
    async fn issue_types(&self, project_key: &str) -> Result<Vec<String>>;
    async fn issue_by_key(&self, key: &str) -> Result<Option<IssueSummary>>;
}

#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>>;
    /// `assignee = currentUser() AND resolution = Unresolved`. Requires a connection.
    async fn my_issues(&self) -> Result<Vec<IssueSummary>>;
    /// Raw user JQL. Requires a connection.
    async fn search_issues(&self, jql: &str) -> Result<Vec<IssueSummary>>;
}

#[async_trait]
pub trait ConflictRepository: Send + Sync {
    async fn conflict_issues(&self) -> Result<Vec<IssueSummary>>;
    async fn conflict_count(&self) -> Result<usize>;
    async fn resolve_conflict_use_local(&self, key: &str) -> Result<()>;
    async fn resolve_conflict_use_remote(&self, key: &str) -> Result<()>;
}

#[async_trait]
pub trait SyncStatusRepository: Send + Sync {
    async fn sync_state(&self) -> Result<SyncState>;
    async fn sync_log(&self, limit: usize, filter: SyncLogFilter) -> Result<Vec<SyncLogEntry>>;
}

#[async_trait]
pub trait SyncExecutor: Send + Sync {
    async fn sync_pull(&self) -> Result<()>;
    async fn sync_push(&self) -> Result<()>;
}

pub trait AppRepository:
    BoardRepository + IssueRepository + ConflictRepository + SyncStatusRepository + MutationRepository
{
}

impl<T> AppRepository for T where
    T: BoardRepository
        + IssueRepository
        + ConflictRepository
        + SyncStatusRepository
        + MutationRepository
{
}
