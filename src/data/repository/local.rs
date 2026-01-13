use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use gouqi::Board;
use log::error;

use crate::{
    client::jira::BoardConfig,
    config::{ProfileConfig, SyncMode},
    data::{
        BoardSummary, CommentSnapshot, IssueSnapshot, IssueSummary, SqliteRepository,
        SqliteRepositoryConfig, SyncLogEntry, SyncLogFilter, SyncState, diff_comment, diff_issue,
        repository::{AppRepository, CommandRepository, QueryRepository, jira::JiraRepository},
    },
};

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
        let profile_id = profile
            .map(|profile| profile.id.clone())
            .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());
        let cache = SqliteRepository::connect(cache_cfg, profile_id.clone()).await?;
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
            profile_id,
        })
    }

    pub fn with_profile(&self, profile: Option<&ProfileConfig>) -> Result<Self> {
        let sync_mode = profile
            .map(|profile| profile.sync_mode())
            .unwrap_or(SyncMode::Cache);
        let profile_id = profile
            .map(|profile| profile.id.clone())
            .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());
        let remote = match profile {
            Some(profile) if sync_mode == SyncMode::Online => {
                Some(Arc::new(JiraRepository::new(&profile.jira)?))
            }
            None => None,
            _ => None,
        };
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

    pub async fn sync_pull(&self) -> Result<()> {
        let Some(remote) = &self.remote else {
            self.cache
                .log_sync_event(
                    "pull",
                    "error",
                    Some("sync pull requires online mode"),
                    Some(self.profile_id.as_str()),
                )
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
                for issue in &issues {
                    let local = self.cache.fetch_issue(&issue.key).await?;
                    let remote_snapshot = IssueSnapshot::from(issue);
                    let mut has_conflict = false;

                    if let Some(local_issue) = &local {
                        if local_issue.dirty {
                            let diffs = diff_issue(local_issue, &remote_snapshot);
                            if !diffs.is_empty() {
                                has_conflict = true;
                            }
                        }

                        let mut comment_conflict = false;
                        if local_issue.comments.iter().any(|c| c.dirty) {
                            let mut remote_by_id = std::collections::HashMap::new();
                            for comment in &remote_snapshot.comments {
                                remote_by_id.insert(comment.id.as_str(), comment);
                            }
                            for local_comment in &local_issue.comments {
                                if !local_comment.dirty {
                                    continue;
                                }
                                match remote_by_id.get(local_comment.id.as_str()) {
                                    Some(remote_comment) => {
                                        if !diff_comment(local_comment, remote_comment).is_empty() {
                                            comment_conflict = true;
                                            let snapshot = serde_json::to_string(remote_comment)
                                                .unwrap_or_default();
                                            if !snapshot.is_empty() {
                                                self.cache
                                                    .mark_comment_conflict(
                                                        &local_comment.id,
                                                        &snapshot,
                                                    )
                                                    .await?;
                                            }
                                        } else {
                                            self.cache
                                                .clear_comment_dirty(&local_comment.id)
                                                .await?;
                                        }
                                    }
                                    None => {
                                        comment_conflict = true;
                                        let missing = CommentSnapshot {
                                            id: local_comment.id.clone(),
                                            author: String::new(),
                                            body: String::new(),
                                            created_at: None,
                                            updated_at: None,
                                        };
                                        let snapshot =
                                            serde_json::to_string(&missing).unwrap_or_default();
                                        if !snapshot.is_empty() {
                                            self.cache
                                                .mark_comment_conflict(&local_comment.id, &snapshot)
                                                .await?;
                                        }
                                    }
                                }
                            }
                        }

                        if has_conflict || comment_conflict {
                            let snapshot =
                                serde_json::to_string(&remote_snapshot).unwrap_or_default();
                            if !snapshot.is_empty() {
                                self.cache
                                    .mark_issue_conflict(&issue.key, &snapshot)
                                    .await?;
                            }
                            continue;
                        }

                        if local_issue.dirty {
                            self.cache.clear_issue_dirty(&issue.key).await?;
                        }
                    }

                    self.cache
                        .upsert_issues(std::slice::from_ref(issue))
                        .await?;
                }
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.cache
                    .log_sync_event("pull", "success", None, Some(self.profile_id.as_str()))
                    .await?;
                Ok(())
            }
            Err(err) => {
                self.cache
                    .log_sync_event(
                        "pull",
                        "error",
                        Some(&err.to_string()),
                        Some(self.profile_id.as_str()),
                    )
                    .await?;
                Err(err)
            }
        }
    }

    pub async fn sync_push(&self) -> Result<()> {
        let Some(remote) = &self.remote else {
            self.cache
                .log_sync_event(
                    "push",
                    "error",
                    Some("sync push requires online mode"),
                    Some(self.profile_id.as_str()),
                )
                .await?;
            return Err(eyre!("Sync push requires online mode"));
        };

        // Cleanup old completed outbox entries (older than 7 days)
        if let Err(err) = self.cache.cleanup_old_outbox(7).await {
            error!("Failed to cleanup old outbox entries: {}", err);
        }

        // Fetch pending outbox items
        let items = self
            .cache
            .fetch_pending_outbox(super::sqlite::OUTBOX_BATCH_SIZE)
            .await?;

        for item in items {
            // Check max attempts
            if item.attempts >= super::sqlite::MAX_OUTBOX_ATTEMPTS {
                self.cache
                    .mark_outbox_failed(&item.id, "max attempts reached")
                    .await?;
                continue;
            }

            // Check for conflicts
            let conflict = match item.entity_type.as_str() {
                "issue" => self.cache.issue_conflict(&item.entity_id).await?,
                "comment" => self.cache.comment_conflict(&item.entity_id).await?,
                _ => false,
            };
            if conflict {
                self.cache
                    .mark_outbox_conflict(&item.id, Some("conflict"))
                    .await?;
                continue;
            }

            self.cache.mark_outbox_processing(&item.id).await?;

            // Push to Jira based on entity type
            let push_result = match item.entity_type.as_str() {
                "issue" => {
                    self.push_issue_to_jira(&item.entity_id, &item.change_set, remote)
                        .await
                }
                "comment" => {
                    self.push_comment_to_jira(&item.entity_id, &item.change_set, remote)
                        .await
                }
                _ => Ok(()),
            };

            match push_result {
                Ok(()) => {
                    self.cache.mark_outbox_done(&item.id).await?;
                }
                Err(err) => {
                    self.cache
                        .mark_outbox_failed(&item.id, &err.to_string())
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn push_issue_to_jira(
        &self,
        issue_key: &str,
        change_set: &str,
        remote: &JiraRepository,
    ) -> Result<()> {
        // Parse the change_set to determine action
        let change: serde_json::Value = serde_json::from_str(change_set)?;
        let action = change
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("Missing action in change_set"))?;

        // Fetch the issue from the cache - if not found, issue was deleted locally
        let Some(issue) = self.cache.fetch_issue(issue_key).await? else {
            return Err(eyre!(
                "Issue {} not found in cache (may have been deleted)",
                issue_key
            ));
        };

        // Get estimation field ID from board config if available
        let estimation_field_id = if let Some(sprint_id) = issue.sprint_id {
            if let Some(board_id) = self.cache.get_board_id_by_sprint(sprint_id).await? {
                self.cache
                    .board_config(board_id)
                    .await
                    .ok()
                    .map(|config| config.estimation.field_id().to_string())
            } else {
                None
            }
        } else {
            None
        };

        if action == "create" && issue_key.starts_with("TEMP-") {
            // Create new issue in Jira
            let new_key = remote
                .create_issue(&issue, estimation_field_id.as_deref())
                .await?;

            // Update key in database (TEMP-xxx -> PROJ-123)
            self.cache.update_issue_key(issue_key, &new_key).await?;

            // Clear dirty flag
            self.cache.clear_issue_dirty(&new_key).await?;
        } else {
            // Update existing issue in Jira
            remote
                .update_issue(&issue, estimation_field_id.as_deref())
                .await?;

            // Clear dirty flag
            self.cache.clear_issue_dirty(issue_key).await?;
        }

        Ok(())
    }

    async fn push_comment_to_jira(
        &self,
        comment_id: &str,
        change_set: &str,
        remote: &JiraRepository,
    ) -> Result<()> {
        // Parse the change_set to determine action
        let change: serde_json::Value = serde_json::from_str(change_set)?;
        let action = change
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("Missing action in change_set"))?;

        // Fetch the comment from cache
        let Some(comment) = self.cache.fetch_comment(comment_id).await? else {
            return Err(eyre!(
                "Comment {} not found in cache (may have been deleted)",
                comment_id
            ));
        };

        if action == "create" && comment_id.starts_with("TEMP-") {
            // Create new comment in Jira
            let new_id = remote
                .create_comment(&comment.issue_key, &comment.body)
                .await?;

            // Update comment ID in database (TEMP-xxx -> real ID)
            self.cache.update_comment_id(comment_id, &new_id).await?;

            // Clear dirty flag
            self.cache.clear_comment_dirty(&new_id).await?;
        } else {
            // Update existing comment in Jira
            remote
                .update_comment(&comment.issue_key, comment_id, &comment.body)
                .await?;

            // Clear dirty flag
            self.cache.clear_comment_dirty(comment_id).await?;
        }

        Ok(())
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
        let Some(issue) = self.cache.fetch_issue(key).await? else {
            return Err(eyre!("Issue {} not found", key));
        };
        let snapshot = issue
            .remote_snapshot
            .ok_or_else(|| eyre!("Remote snapshot missing for issue {}", key))?;
        let remote: IssueSnapshot = serde_json::from_str(snapshot.as_str())?;
        self.cache.resolve_issue_use_remote(key, &remote).await
    }

    async fn sync_log(&self, limit: usize, filter: SyncLogFilter) -> Result<Vec<SyncLogEntry>> {
        self.cache.sync_log(limit, filter).await
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
