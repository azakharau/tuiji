#[derive(Debug, Clone)]
pub struct IssueSummary {
    pub key: String,
    pub summary: String,
    pub epic: Option<String>,
    pub status: String,
    pub issue_type: String,
    pub assignee: String,
    pub priority: String,
    pub story_points: Option<f64>,
    pub project_key: Option<String>,
    pub sprint_id: Option<i64>,
    pub updated_at: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct BoardSummary {
    pub id: u64,
    pub name: String,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OutboxCommand {
    CreateIssue { summary: String },
    UpdateIssue { key: String },
}

#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub last_full_sync: Option<std::time::SystemTime>,
    pub last_pull: Option<std::time::SystemTime>,
    pub last_push: Option<std::time::SystemTime>,
}
