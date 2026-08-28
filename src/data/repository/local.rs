use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{
    Result,
    eyre::{Report, eyre},
};

use crate::{
    client::jira::write::WriteError,
    config::ProfileConfig,
    data::{
        BoardConfig, BoardSummary, IssueSummary, OutboxCommand, SqliteRepository,
        SqliteRepositoryConfig, SyncLogEntry, SyncLogFilter, SyncState,
        model::{IssueMutation, OutboxChange, TransitionOptions},
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

    /// Cached transitions stay valid only while the issue sits in the status
    /// they were fetched under. Jira scopes the list to the current status, so
    /// offering a list captured under another status would queue a transition
    /// Jira is going to reject.
    async fn usable_cached_transitions(&self, key: &str) -> Result<Option<TransitionOptions>> {
        let Some((cached_status, choices)) = self.cache.cached_transitions(key).await? else {
            return Ok(None);
        };
        let Some(issue) = self.cache.issue_by_key(key).await? else {
            return Ok(None);
        };
        if issue.status != cached_status {
            return Ok(None);
        }
        Ok(Some(TransitionOptions {
            choices,
            from_cache: true,
        }))
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

    async fn available_transitions(&self, key: &str) -> Result<TransitionOptions> {
        let remote_error = match self.remote.as_deref() {
            Some(remote) => match remote.list_transitions(key).await {
                Ok(choices) => {
                    if let Some(issue) = self.cache.issue_by_key(key).await? {
                        self.cache
                            .cache_transitions(key, &issue.status, &choices)
                            .await?;
                    }
                    return Ok(TransitionOptions {
                        choices,
                        from_cache: false,
                    });
                }
                // Jira answered. A rejection or a conflict is that answer, and
                // hiding it behind a cached list labelled "unreachable" would
                // misreport a bad token or a lost issue as an offline blip.
                Err(error) => match error {
                    WriteError::Transient(_) => eyre!(error),
                    answered => return Err(eyre!(answered)),
                },
            },
            None => eyre!("Loading available transitions requires an active Jira connection"),
        };

        match self.usable_cached_transitions(key).await? {
            Some(options) => Ok(options),
            None => Err(remote_error),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::JiraConfig, data::model::TransitionChoice};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn issue(key: &str, status: &str) -> IssueSummary {
        IssueSummary {
            key: key.to_string(),
            summary: format!("Summary for {key}"),
            epic: None,
            status: status.to_string(),
            issue_type: "Task".to_string(),
            assignee: "Alex".to_string(),
            priority: "Medium".to_string(),
            story_points: None,
            project_key: Some("SMK".to_string()),
            sprint_id: None,
            updated_at: None,
            comments: Vec::new(),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: None,
            reporter: None,
            creator: None,
            created_at: None,
            resolution_date: None,
            resolution: None,
            labels: Vec::new(),
            fix_versions: Vec::new(),
            parent_key: None,
            environment: None,
            time_estimate: None,
            time_spent: None,
            time_remaining: None,
            custom_fields: None,
        }
    }

    fn choices() -> Vec<TransitionChoice> {
        vec![TransitionChoice {
            id: "31".to_string(),
            name: "Done".to_string(),
            to_status: "Done".to_string(),
        }]
    }

    /// A hub without a profile has no remote, which is exactly the offline case
    /// the transition cache exists for.
    async fn offline_hub() -> (RepositoryHub, std::path::PathBuf) {
        let db_path = std::env::temp_dir().join(format!("tuiji-hub-{}.db", uuid::Uuid::now_v7()));
        let hub = RepositoryHub::connect(
            SqliteRepositoryConfig {
                db_path: db_path.clone(),
            },
            None,
        )
        .await
        .unwrap();
        (hub, db_path)
    }

    #[tokio::test]
    async fn transitions_should_fall_back_to_the_cached_list_when_jira_is_unreachable() {
        let (hub, db_path) = offline_hub().await;
        hub.cache
            .upsert_issues_impl(&[issue("SMK-1", "TODO")])
            .await
            .unwrap();
        hub.cache
            .cache_transitions("SMK-1", "TODO", &choices())
            .await
            .unwrap();

        let options = hub.available_transitions("SMK-1").await.unwrap();

        assert_eq!(options.choices, choices());
        assert!(
            options.from_cache,
            "the picker must be able to tell the user this list was not just confirmed by Jira"
        );

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn cached_transitions_should_be_dropped_once_the_issue_left_that_status() {
        let (hub, db_path) = offline_hub().await;
        hub.cache
            .upsert_issues_impl(&[issue("SMK-1", "TODO")])
            .await
            .unwrap();
        hub.cache
            .cache_transitions("SMK-1", "TODO", &choices())
            .await
            .unwrap();
        hub.cache
            .set_issue_status("SMK-1", "In Progress")
            .await
            .unwrap();

        let error = hub
            .available_transitions("SMK-1")
            .await
            .expect_err("a list captured under another status must not be offered")
            .to_string();

        assert!(error.contains("active Jira connection"), "got: {error}");

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn transitions_should_report_the_connection_error_when_nothing_is_cached() {
        let (hub, db_path) = offline_hub().await;
        hub.cache
            .upsert_issues_impl(&[issue("SMK-1", "TODO")])
            .await
            .unwrap();

        let error = hub
            .available_transitions("SMK-1")
            .await
            .expect_err("without a cached list there is nothing honest to show")
            .to_string();

        assert!(error.contains("active Jira connection"), "got: {error}");

        let _ = std::fs::remove_file(db_path);
    }

    async fn hub_with_jira(base_url: &str) -> (RepositoryHub, std::path::PathBuf) {
        let profile = ProfileConfig {
            id: "smoke".to_string(),
            name: "Smoke".to_string(),
            jira: JiraConfig {
                base_url: base_url.to_string(),
                username: "user@example.com".to_string(),
                api_token: "token".to_string(),
                api_token_command: None,
            },
        };
        let db_path = std::env::temp_dir().join(format!("tuiji-hub-{}.db", uuid::Uuid::now_v7()));
        let hub = RepositoryHub::connect(
            SqliteRepositoryConfig {
                db_path: db_path.clone(),
            },
            Some(&profile),
        )
        .await
        .unwrap();
        (hub, db_path)
    }

    async fn jira_answering(status: u16) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/SMK-1/transitions"))
            .respond_with(ResponseTemplate::new(status))
            .mount(&server)
            .await;
        server
    }

    async fn cached_hub(server: &MockServer) -> (RepositoryHub, std::path::PathBuf) {
        let (hub, db_path) = hub_with_jira(&server.uri()).await;
        hub.cache
            .upsert_issues_impl(&[issue("SMK-1", "TODO")])
            .await
            .unwrap();
        hub.cache
            .cache_transitions("SMK-1", "TODO", &choices())
            .await
            .unwrap();
        (hub, db_path)
    }

    #[tokio::test]
    async fn an_unreachable_jira_should_fall_back_to_the_cached_list() {
        let server = jira_answering(503).await;
        let (hub, db_path) = cached_hub(&server).await;

        let options = hub.available_transitions("SMK-1").await.unwrap();

        assert!(options.from_cache);
        assert_eq!(options.choices, choices());

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn a_rejection_from_jira_should_not_be_masked_by_the_cached_list() {
        let server = jira_answering(401).await;
        let (hub, db_path) = cached_hub(&server).await;

        let error = hub
            .available_transitions("SMK-1")
            .await
            .expect_err("Jira answered, so its rejection is the answer the user needs")
            .to_string();

        assert!(
            !error.contains("active Jira connection"),
            "a rejection must not be reported as being offline, got: {error}"
        );

        let _ = std::fs::remove_file(db_path);
    }
}
