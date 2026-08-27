use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::Result;

use crate::{
    client::jira::{
        JiraClient,
        write::{JiraWriteClient, WriteError},
    },
    config::JiraConfig,
    data::{
        model::{BoardConfig, IssueDraft, IssueSummary, OutboxChange, SyncState, TransitionChoice},
        repository::{CommandRepository, QueryRepository},
    },
};

mod comments;
mod custom_fields;
mod issue_mapping;

use issue_mapping::map_issue;

pub struct JiraRepository {
    client: Arc<JiraClient>,
    writer: Arc<JiraWriteClient>,
}

impl JiraRepository {
    pub fn new(cfg: &JiraConfig) -> Result<Self> {
        let token = cfg.resolve_token()?;
        let client = JiraClient::new(cfg.base_url.as_str(), cfg.username.as_str(), token.as_str())?;
        let writer =
            JiraWriteClient::new(cfg.base_url.as_str(), cfg.username.as_str(), token.as_str())?;
        Ok(Self {
            client: Arc::new(client),
            writer: Arc::new(writer),
        })
    }

    pub async fn list_boards(&self) -> Result<Vec<gouqi::Board>> {
        Ok(self.client.get_boards().await?)
    }

    pub async fn apply_outbox_change(
        &self,
        _entity_type: &str,
        entity_id: &str,
        change_set: &str,
    ) -> std::result::Result<(), WriteError> {
        let change = serde_json::from_str::<OutboxChange>(change_set).map_err(|error| {
            WriteError::Permanent(format!("Invalid outbox change set: {error}"))
        })?;

        match change {
            OutboxChange::Fields { fields } => {
                self.writer
                    .edit_fields(entity_id, serde_json::json!({ "fields": fields }))
                    .await
            }
            OutboxChange::Transition { id, .. } => {
                self.writer.trigger_transition(entity_id, &id).await
            }
            OutboxChange::Comment { issue_key, body } => {
                self.writer.add_comment(&issue_key, &body).await
            }
            OutboxChange::Assignee { account_id } => {
                self.writer.set_assignee(entity_id, &account_id).await
            }
        }
    }

    pub async fn create_issue(
        &self,
        draft: &IssueDraft,
    ) -> std::result::Result<String, WriteError> {
        self.writer.create_issue(draft).await
    }

    pub async fn list_transitions(
        &self,
        key: &str,
    ) -> std::result::Result<Vec<TransitionChoice>, WriteError> {
        self.writer.list_transitions(key).await
    }

    pub async fn issue_types(
        &self,
        project_key: &str,
    ) -> std::result::Result<Vec<String>, WriteError> {
        self.writer.issue_types(project_key).await
    }

    pub async fn myself(&self) -> std::result::Result<(String, String), WriteError> {
        self.writer.myself().await
    }

    pub async fn my_issues(&self) -> Result<Vec<IssueSummary>> {
        self.search_issues(
            "assignee = currentUser() AND resolution = Unresolved ORDER BY updated DESC",
        )
        .await
    }

    pub async fn search_issues(&self, jql: &str) -> Result<Vec<IssueSummary>> {
        Ok(self
            .client
            .search_issues(jql)
            .await?
            .iter()
            .map(|issue| map_issue(issue, None))
            .collect())
    }
}

#[async_trait]
impl QueryRepository for JiraRepository {
    async fn board_config(&self, board_id: u64) -> Result<BoardConfig> {
        Ok(self.client.get_board_config(board_id).await?)
    }

    async fn current_sprint_issues(&self, board_id: u64) -> Result<Vec<IssueSummary>> {
        let sprint = self.client.get_current_sprint(board_id).await?;
        let sprint_id = Some(sprint.id as i64);
        let jira_issues = self.client.get_sprint_issues(sprint.id).await?;
        Ok(jira_issues
            .into_iter()
            .map(|issue| map_issue(&issue, sprint_id))
            .collect())
    }

    async fn sync_state(&self) -> Result<SyncState> {
        Ok(SyncState::default())
    }
}

#[async_trait]
impl CommandRepository for JiraRepository {}
