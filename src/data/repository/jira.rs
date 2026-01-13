use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::Result;
use serde_json::Value;

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

    pub async fn create_issue(
        &self,
        issue: &IssueSummary,
        estimation_field_id: Option<&str>,
    ) -> Result<String> {
        // Convert IssueSummary to BTreeMap of fields for Jira API
        let fields = issue_summary_to_create_fields(issue, estimation_field_id)?;
        let response = self.client.create_issue(fields).await?;
        Ok(response.key)
    }

    pub async fn update_issue(
        &self,
        issue: &IssueSummary,
        estimation_field_id: Option<&str>,
    ) -> Result<()> {
        // Convert IssueSummary to gouqi EditIssue format (BTreeMap of fields)
        let fields = issue_summary_to_fields(issue, estimation_field_id)?;
        self.client.update_issue(issue.key.as_str(), fields).await?;
        Ok(())
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

// Convert IssueSummary to BTreeMap of fields for create
fn issue_summary_to_create_fields(
    issue: &IssueSummary,
    estimation_field_id: Option<&str>,
) -> Result<BTreeMap<String, Value>> {
    let mut fields = BTreeMap::new();

    // Extract project key from issue key or use provided project_key
    let project_key = issue
        .project_key
        .clone()
        .unwrap_or_else(|| issue.key.split('-').next().unwrap_or("UNKNOWN").to_string());

    // Required fields for issue creation
    fields.insert(
        "project".to_string(),
        serde_json::json!({ "key": project_key }),
    );
    fields.insert("summary".to_string(), Value::String(issue.summary.clone()));
    fields.insert(
        "issuetype".to_string(),
        serde_json::json!({ "name": issue.issue_type }),
    );

    // Priority
    fields.insert(
        "priority".to_string(),
        serde_json::json!({ "name": issue.priority }),
    );

    // Assignee (skip if Unassigned)
    if issue.assignee != "Unassigned" {
        fields.insert(
            "assignee".to_string(),
            serde_json::json!({ "name": issue.assignee }),
        );
    }

    // Optional fields
    if let Some(ref desc) = issue.description {
        fields.insert("description".to_string(), Value::String(desc.clone()));
    }

    if let Some(ref reporter) = issue.reporter {
        fields.insert(
            "reporter".to_string(),
            serde_json::json!({ "name": reporter }),
        );
    }

    if !issue.labels.is_empty() {
        fields.insert("labels".to_string(), serde_json::json!(issue.labels));
    }

    if let Some(ref env) = issue.environment {
        fields.insert("environment".to_string(), Value::String(env.clone()));
    }

    // Story points (custom field - use provided field_id or default)
    if let Some(story_points) = issue.story_points {
        let field_id = estimation_field_id.unwrap_or("customfield_10002");
        fields.insert(
            field_id.to_string(),
            Value::Number(serde_json::Number::from_f64(story_points).unwrap()),
        );
    }

    // Parent issue (for subtasks)
    if let Some(ref parent) = issue.parent_key {
        fields.insert("parent".to_string(), serde_json::json!({ "key": parent }));
    }

    // Merge any additional custom fields from the issue
    if let Some(ref custom_json) = issue.custom_fields {
        if let Ok(custom_map) = serde_json::from_str::<BTreeMap<String, Value>>(custom_json) {
            for (key, value) in custom_map {
                if !fields.contains_key(&key) {
                    fields.insert(key, value);
                }
            }
        }
    }

    Ok(fields)
}

// Convert IssueSummary to BTreeMap of fields for update
fn issue_summary_to_fields(
    issue: &IssueSummary,
    estimation_field_id: Option<&str>,
) -> Result<BTreeMap<String, Value>> {
    let mut fields = BTreeMap::new();

    // Basic fields
    fields.insert("summary".to_string(), Value::String(issue.summary.clone()));

    if let Some(ref desc) = issue.description {
        fields.insert("description".to_string(), Value::String(desc.clone()));
    }

    fields.insert(
        "issuetype".to_string(),
        serde_json::json!({ "name": issue.issue_type }),
    );
    fields.insert(
        "priority".to_string(),
        serde_json::json!({ "name": issue.priority }),
    );

    // Assignee
    if issue.assignee != "Unassigned" {
        fields.insert(
            "assignee".to_string(),
            serde_json::json!({ "name": issue.assignee }),
        );
    }

    // Optional fields
    if let Some(ref env) = issue.environment {
        fields.insert("environment".to_string(), Value::String(env.clone()));
    }

    if let Some(story_points) = issue.story_points {
        let field_id = estimation_field_id.unwrap_or("customfield_10002");
        fields.insert(
            field_id.to_string(),
            Value::Number(serde_json::Number::from_f64(story_points).unwrap()),
        );
    }

    if !issue.labels.is_empty() {
        fields.insert("labels".to_string(), serde_json::json!(issue.labels));
    }

    Ok(fields)
}

impl JiraRepository {
    /// Create a comment in Jira
    pub async fn create_comment(&self, issue_key: &str, body: &str) -> Result<String> {
        let comment_id = self.client.create_comment(issue_key, body).await?;
        Ok(comment_id)
    }

    /// Update a comment in Jira
    pub async fn update_comment(
        &self,
        issue_key: &str,
        comment_id: &str,
        body: &str,
    ) -> Result<()> {
        self.client
            .update_comment(issue_key, comment_id, body)
            .await?;
        Ok(())
    }
}
