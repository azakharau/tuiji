use serde::{Deserialize, Serialize};

use crate::data::model::{IssueComment, IssueSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSnapshot {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub issue_type: String,
    pub assignee: String,
    pub priority: String,
    pub epic: Option<String>,
    pub story_points: Option<f64>,
    pub project_key: Option<String>,
    pub sprint_id: Option<i64>,
    pub comments: Vec<CommentSnapshot>,

    // New fields
    pub description: Option<String>,
    pub reporter: Option<String>,
    pub creator: Option<String>,
    pub created_at: Option<i64>,
    pub resolution_date: Option<i64>,
    pub resolution: Option<String>,
    pub labels: Option<String>,
    pub fix_versions: Option<String>,
    pub parent_key: Option<String>,
    pub environment: Option<String>,
    pub time_estimate: Option<String>,
    pub time_spent: Option<String>,
    pub time_remaining: Option<String>,
    pub custom_fields: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentSnapshot {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub field: &'static str,
    pub local: String,
    pub remote: String,
}

impl From<&IssueSummary> for IssueSnapshot {
    fn from(issue: &IssueSummary) -> Self {
        Self {
            key: issue.key.clone(),
            summary: issue.summary.clone(),
            status: issue.status.clone(),
            issue_type: issue.issue_type.clone(),
            assignee: issue.assignee.clone(),
            priority: issue.priority.clone(),
            epic: issue.epic.clone(),
            story_points: issue.story_points,
            project_key: issue.project_key.clone(),
            sprint_id: issue.sprint_id,
            comments: issue.comments.iter().map(CommentSnapshot::from).collect(),

            // New fields
            description: issue.description.clone(),
            reporter: issue.reporter.clone(),
            creator: issue.creator.clone(),
            created_at: system_time_to_ts(issue.created_at),
            resolution_date: system_time_to_ts(issue.resolution_date),
            resolution: issue.resolution.clone(),
            labels: serialize_json_array(&issue.labels),
            fix_versions: serialize_json_array(&issue.fix_versions),
            parent_key: issue.parent_key.clone(),
            environment: issue.environment.clone(),
            time_estimate: issue.time_estimate.clone(),
            time_spent: issue.time_spent.clone(),
            time_remaining: issue.time_remaining.clone(),
            custom_fields: issue.custom_fields.clone(),
        }
    }
}

impl From<&IssueComment> for CommentSnapshot {
    fn from(comment: &IssueComment) -> Self {
        Self {
            id: comment.id.clone(),
            author: comment.author.clone(),
            body: comment.body.clone(),
            created_at: comment.created_at.clone(),
            updated_at: comment.updated_at.clone(),
        }
    }
}

pub fn diff_issue(local: &IssueSummary, remote: &IssueSnapshot) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();
    push_if_diff("summary", &local.summary, &remote.summary, &mut diffs);
    push_if_diff("status", &local.status, &remote.status, &mut diffs);
    push_if_diff(
        "issue_type",
        &local.issue_type,
        &remote.issue_type,
        &mut diffs,
    );
    push_if_diff("assignee", &local.assignee, &remote.assignee, &mut diffs);
    push_if_diff("priority", &local.priority, &remote.priority, &mut diffs);
    push_if_diff_opt(
        "epic",
        local.epic.as_deref(),
        remote.epic.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt_f64(
        "story_points",
        local.story_points,
        remote.story_points,
        &mut diffs,
    );
    push_if_diff_opt(
        "project_key",
        local.project_key.as_deref(),
        remote.project_key.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt_i64("sprint_id", local.sprint_id, remote.sprint_id, &mut diffs);
    diffs
}

pub fn diff_comment(local: &IssueComment, remote: &CommentSnapshot) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();
    push_if_diff("author", &local.author, &remote.author, &mut diffs);
    push_if_diff("body", &local.body, &remote.body, &mut diffs);
    push_if_diff_opt(
        "created_at",
        local.created_at.as_deref(),
        remote.created_at.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "updated_at",
        local.updated_at.as_deref(),
        remote.updated_at.as_deref(),
        &mut diffs,
    );
    diffs
}

fn push_if_diff(field: &'static str, local: &str, remote: &str, diffs: &mut Vec<DiffEntry>) {
    if local != remote {
        diffs.push(DiffEntry {
            field,
            local: local.to_string(),
            remote: remote.to_string(),
        });
    }
}

fn push_if_diff_opt(
    field: &'static str,
    local: Option<&str>,
    remote: Option<&str>,
    diffs: &mut Vec<DiffEntry>,
) {
    let local = local.unwrap_or_default();
    let remote = remote.unwrap_or_default();
    push_if_diff(field, local, remote, diffs);
}

fn push_if_diff_opt_f64(
    field: &'static str,
    local: Option<f64>,
    remote: Option<f64>,
    diffs: &mut Vec<DiffEntry>,
) {
    let local = local.map(|v| v.to_string()).unwrap_or_default();
    let remote = remote.map(|v| v.to_string()).unwrap_or_default();
    push_if_diff(field, local.as_str(), remote.as_str(), diffs);
}

fn push_if_diff_opt_i64(
    field: &'static str,
    local: Option<i64>,
    remote: Option<i64>,
    diffs: &mut Vec<DiffEntry>,
) {
    let local = local.map(|v| v.to_string()).unwrap_or_default();
    let remote = remote.map(|v| v.to_string()).unwrap_or_default();
    push_if_diff(field, local.as_str(), remote.as_str(), diffs);
}

// Helper function to convert SystemTime to i64 timestamp
fn system_time_to_ts(value: Option<std::time::SystemTime>) -> Option<i64> {
    value.map(|ts| {
        ts.duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    })
}

// Helper function to serialize Vec<String> to JSON string
fn serialize_json_array(value: &[String]) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        serde_json::to_string(value).ok()
    }
}
