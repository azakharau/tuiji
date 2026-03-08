use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};

use crate::{
    client::jira::JiraClient,
    config::JiraConfig,
    data::model::{BoardConfig, IssueSummary, SyncState},
    data::repository::{CommandRepository, QueryRepository},
};

mod comments;
mod custom_fields;
mod issue_mapping;

use issue_mapping::map_issue;

pub struct JiraRepository {
    client: Arc<JiraClient>,
}

impl JiraRepository {
    pub fn new(cfg: &JiraConfig) -> Result<Self> {
        let client = JiraClient::new(
            cfg.base_url.as_str(),
            cfg.username.as_str(),
            cfg.api_token.as_str(),
        )?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn list_boards(&self) -> Result<Vec<gouqi::Board>> {
        Ok(self.client.get_boards().await?)
    }

    pub async fn apply_outbox_change(
        &self,
        entity_type: &str,
        _entity_id: &str,
        _change_set: &str,
    ) -> Result<()> {
        let _client = &self.client;
        Err(eyre!(
            "Jira remote push is not implemented for entity type \"{entity_type}\""
        ))
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
