use async_trait::async_trait;
use color_eyre::Result;

use crate::{
    client::jira::{BoardConfig, JiraClient},
    config::JiraConfig,
    data::model::{IssueSummary, SyncState},
    data::repository::{CommandRepository, QueryRepository},
};

pub struct JiraRepository {
    client: JiraClient,
}

impl JiraRepository {
    pub fn new(cfg: &JiraConfig) -> Result<Self> {
        let client = JiraClient::new(
            cfg.base_url.as_str(),
            cfg.username.as_str(),
            cfg.api_token.as_str(),
        )?;
        Ok(Self { client })
    }

    pub async fn list_boards(&self) -> Result<Vec<gouqi::Board>> {
        Ok(self.client.get_boards().await?)
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
        let jira_issues = self.client.get_current_sprint_issues(board_id).await?;
        let mut issues = Vec::with_capacity(jira_issues.len());
        for issue in jira_issues {
            let key = issue.key.to_string();
            let project_key = key.split('-').next().map(|v| v.to_string());
            let summary = issue.summary().unwrap_or_default();
            let epic = None;
            let status = match issue.status() {
                Some(st) => st.name.to_uppercase(),
                None => "TODO".to_string(),
            };
            let issue_type = match issue.issue_type() {
                Some(it) => it.name,
                None => "Task".to_string(),
            };
            let assignee = match issue.assignee() {
                Some(user) => user.display_name,
                None => "Unassigned".to_string(),
            };
            let priority = match issue.priority() {
                Some(pr) => pr.name,
                None => "Medium".to_string(),
            };
            let story_points = None;
            let updated_at = issue.updated().map(|ts| {
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.unix_timestamp() as u64)
            });

            issues.push(IssueSummary {
                key,
                summary,
                epic,
                status,
                issue_type,
                assignee,
                priority,
                story_points,
                project_key,
                sprint_id,
                updated_at,
            });
        }
        Ok(issues)
    }

    async fn sync_state(&self) -> Result<SyncState> {
        Ok(SyncState::default())
    }
}

#[async_trait]
impl CommandRepository for JiraRepository {}
