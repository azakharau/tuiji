use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{
    Result,
    eyre::{Report, eyre},
};

use crate::{
    config::ProfileConfig,
    data::{
        BoardConfig, BoardSummary, IssueSummary, OutboxCommand, SqliteRepository,
        SqliteRepositoryConfig, SyncLogEntry, SyncLogFilter, SyncState,
        model::{IssueMutation, OutboxChange, TransitionChoice},
        repository::{
            BoardRepository, CommandRepository, ConflictRepository, IssueRepository,
            MutationRepository, QueryRepository, SyncStatusRepository, jira::JiraRepository,
        },
    },
};

mod conflicts;
mod sync;

const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Clone)]
pub struct RepositoryHub {
    cache: Arc<SqliteRepository>,
    remote: Option<Arc<JiraRepository>>,
    my_identity: Arc<tokio::sync::OnceCell<(String, String)>>,
    profile_id: String,
}

impl RepositoryHub {
    pub async fn connect(
        cache_cfg: SqliteRepositoryConfig,
        profile: Option<&ProfileConfig>,
    ) -> Result<Self> {
        let profile_id = resolve_profile_id(profile);
        let cache = SqliteRepository::connect(cache_cfg, profile_id.clone()).await?;
        let remote = build_remote(profile)?;
        Ok(Self {
            cache: Arc::new(cache),
            remote,
            my_identity: Arc::new(tokio::sync::OnceCell::new()),
            profile_id,
        })
    }

    pub fn with_profile(&self, profile: Option<&ProfileConfig>) -> Result<Self> {
        let profile_id = resolve_profile_id(profile);
        let remote = build_remote(profile)?;
        Ok(Self {
            cache: Arc::new(self.cache.with_profile(profile_id.clone())),
            remote,
            my_identity: Arc::new(tokio::sync::OnceCell::new()),
            profile_id,
        })
    }

    pub fn cache(&self) -> Arc<SqliteRepository> {
        Arc::clone(&self.cache)
    }

    pub async fn requeue_failed_outbox(&self) -> Result<u64> {
        self.cache.requeue_failed_outbox().await
    }

    fn require_remote(&self, operation: &str) -> Result<&JiraRepository> {
        self.remote
            .as_deref()
            .ok_or_else(|| eyre!("{operation} requires an active Jira connection"))
    }

    async fn resolve_my_identity(&self) -> Result<&(String, String)> {
        let remote = self.require_remote("Assigning an issue to yourself")?;
        self.my_identity
            .get_or_try_init(|| async { Ok::<_, Report>(remote.myself().await?) })
            .await
    }
}

#[async_trait]
impl BoardRepository for RepositoryHub {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig> {
        self.cache.board_config(board_id).await
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
}

#[async_trait]
impl IssueRepository for RepositoryHub {
    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>> {
        self.cache.current_sprint_issues(board_id).await
    }

    async fn my_issues(&self) -> Result<Vec<IssueSummary>> {
        let issues = self
            .require_remote("Loading My Issues")?
            .my_issues()
            .await?;
        self.cache.upsert_issues(&issues).await?;
        Ok(issues)
    }

    async fn search_issues(&self, jql: &str) -> Result<Vec<IssueSummary>> {
        let issues = self
            .require_remote("Searching issues")?
            .search_issues(jql)
            .await?;
        self.cache.upsert_issues(&issues).await?;
        Ok(issues)
    }
}

#[async_trait]
impl MutationRepository for RepositoryHub {
    async fn apply_mutation(&self, mutation: IssueMutation) -> Result<()> {
        match mutation {
            IssueMutation::Create(draft) => {
                self.require_remote("Creating an issue")?
                    .create_issue(&draft)
                    .await?;
            }
            IssueMutation::Patch { key, patch } => {
                if patch.is_empty() {
                    return Ok(());
                }

                let mut fields = serde_json::Map::new();
                if let Some(summary) = &patch.summary {
                    fields.insert("summary".to_string(), serde_json::json!(summary));
                }
                if let Some(description) = &patch.description {
                    fields.insert(
                        "description".to_string(),
                        serde_json::to_value(gouqi::AdfDocument::from_text(description))?,
                    );
                }
                if let Some(priority) = &patch.priority {
                    fields.insert(
                        "priority".to_string(),
                        serde_json::json!({ "name": priority }),
                    );
                }

                self.cache.apply_issue_patch(&key, &patch).await?;
                self.cache
                    .enqueue_outbox(OutboxCommand::issue(
                        &key,
                        serde_json::to_string(&OutboxChange::Fields { fields })?,
                    ))
                    .await?;
            }
            IssueMutation::Comment { key, body } => {
                let id = format!("local:{}", uuid::Uuid::now_v7());
                self.cache.insert_local_comment(&key, &id, &body).await?;
                self.cache
                    .enqueue_outbox(OutboxCommand::comment(
                        &id,
                        serde_json::to_string(&OutboxChange::Comment {
                            issue_key: key,
                            body,
                        })?,
                    ))
                    .await?;
            }
            IssueMutation::Transition { key, id, to_status } => {
                self.cache.set_issue_status(&key, &to_status).await?;
                self.cache
                    .enqueue_outbox(OutboxCommand::issue(
                        &key,
                        serde_json::to_string(&OutboxChange::Transition { id, to_status })?,
                    ))
                    .await?;
            }
            IssueMutation::AssignToMe { key } => {
                let (account_id, display_name) = self.resolve_my_identity().await?;
                self.cache.set_issue_assignee(&key, display_name).await?;
                self.cache
                    .enqueue_outbox(OutboxCommand::issue(
                        &key,
                        serde_json::to_string(&OutboxChange::Assignee {
                            account_id: account_id.clone(),
                        })?,
                    ))
                    .await?;
            }
        }
        Ok(())
    }

    async fn available_transitions(&self, key: &str) -> Result<Vec<TransitionChoice>> {
        Ok(self
            .require_remote("Loading available transitions")?
            .list_transitions(key)
            .await?)
    }

    async fn issue_types(&self, project_key: &str) -> Result<Vec<String>> {
        Ok(self
            .require_remote("Loading issue types")?
            .issue_types(project_key)
            .await?)
    }

    async fn issue_by_key(&self, key: &str) -> Result<Option<IssueSummary>> {
        self.cache.issue_by_key(key).await
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
        self.resolve_conflict_use_local_impl(key).await
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

fn build_remote(profile: Option<&ProfileConfig>) -> Result<Option<Arc<JiraRepository>>> {
    profile
        .map(|profile| JiraRepository::new(&profile.jira).map(Arc::new))
        .transpose()
}
