use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::Result;

use crate::{
    client::jira::{BoardConfig, JiraClient},
    config::JiraConfig,
    data::model::{IssueComment, IssueSummary, SyncState},
    data::repository::{CommandRepository, QueryRepository},
};

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

            // Extract new fields
            let description = issue.description();
            let reporter = issue.reporter().map(|u| u.display_name);
            let creator = issue.creator().map(|u| u.display_name);
            let created_at = issue.created().map(|ts| {
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.unix_timestamp() as u64)
            });
            let resolution_date = issue.resolution_date().map(|ts| {
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.unix_timestamp() as u64)
            });

            // Extract resolution name from fields (Resolution type doesn't expose name publicly)
            let resolution = issue
                .resolution()
                .and_then(|_| issue.fields.get("resolution"))
                .and_then(|r| r.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            let labels = issue.labels();
            let fix_versions = issue.fix_versions().into_iter().map(|v| v.name).collect();
            let parent_key = issue.parent().map(|p| p.key);
            let environment = issue.environment();

            // Time tracking fields
            let (time_estimate, time_spent, time_remaining) = match issue.timetracking() {
                Some(tt) => (tt.original_estimate, tt.time_spent, tt.remaining_estimate),
                None => (None, None, None),
            };

            // Custom fields - collect all customfield_* into JSON
            let custom_fields = extract_custom_fields(&issue);

            let comments = extract_comments(&issue, &key);
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
                comments,
                dirty: false,
                conflict: false,
                remote_snapshot: None,
                description,
                reporter,
                creator,
                created_at,
                resolution_date,
                resolution,
                labels,
                fix_versions,
                parent_key,
                environment,
                time_estimate,
                time_spent,
                time_remaining,
                custom_fields,
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

fn extract_comments(issue: &gouqi::Issue, issue_key: &str) -> Vec<IssueComment> {
    let Some(value) = issue.fields.get("comment") else {
        return Vec::new();
    };
    let Some(comments) = value.get("comments").and_then(|c| c.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(comments.len());
    for comment in comments {
        let id = comment
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            continue;
        }
        let author = comment
            .get("author")
            .and_then(|a| a.get("displayName"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let body = match comment.get("body") {
            Some(body) if body.is_string() => body.as_str().unwrap_or_default().to_string(),
            Some(body) => serde_json::to_string(body).unwrap_or_default(),
            None => String::new(),
        };
        let created_at = comment
            .get("created")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let updated_at = comment
            .get("updated")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        out.push(IssueComment {
            id,
            issue_key: issue_key.to_string(),
            author,
            body,
            created_at,
            updated_at,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
        });
    }
    out
}

// Extract custom fields from Jira issue
fn extract_custom_fields(issue: &gouqi::Issue) -> Option<String> {
    let mut custom = std::collections::BTreeMap::new();
    for (key, value) in &issue.fields {
        if key.starts_with("customfield_") {
            custom.insert(key.clone(), value.clone());
        }
    }
    if custom.is_empty() {
        None
    } else {
        serde_json::to_string(&custom).ok()
    }
}
