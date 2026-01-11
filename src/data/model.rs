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
    pub comments: Vec<IssueComment>,
    pub dirty: bool,
    pub conflict: bool,
    pub remote_snapshot: Option<String>,

    // === НОВЫЕ ПОЛЯ ===

    // Основное содержимое
    pub description: Option<String>,

    // Авторство
    pub reporter: Option<String>,
    pub creator: Option<String>,

    // Временные метки
    pub created_at: Option<std::time::SystemTime>,
    pub resolution_date: Option<std::time::SystemTime>,

    // Статус и завершение
    pub resolution: Option<String>,

    // Организация
    pub labels: Vec<String>,
    pub fix_versions: Vec<String>,
    pub parent_key: Option<String>,

    // Окружение
    pub environment: Option<String>,

    // Time tracking
    pub time_estimate: Option<String>,
    pub time_spent: Option<String>,
    pub time_remaining: Option<String>,

    // Custom fields (JSON)
    pub custom_fields: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IssueComment {
    pub id: String,
    pub issue_key: String,
    pub author: String,
    pub body: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub dirty: bool,
    pub conflict: bool,
    pub remote_snapshot: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BoardSummary {
    pub id: u64,
    pub name: String,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxEntityType {
    Issue,
    Comment,
}

#[derive(Debug, Clone)]
pub struct OutboxCommand {
    pub entity_type: OutboxEntityType,
    pub entity_id: String,
    pub change_set: String,
}

impl OutboxCommand {
    pub fn issue(entity_id: impl Into<String>, change_set: String) -> Self {
        Self {
            entity_type: OutboxEntityType::Issue,
            entity_id: entity_id.into(),
            change_set,
        }
    }

    pub fn comment(entity_id: impl Into<String>, change_set: String) -> Self {
        Self {
            entity_type: OutboxEntityType::Comment,
            entity_id: entity_id.into(),
            change_set,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub last_full_sync: Option<std::time::SystemTime>,
    pub last_pull: Option<std::time::SystemTime>,
    pub last_push: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncLogFilter {
    All,
    Pull,
    Push,
}

#[derive(Debug, Clone)]
pub struct SyncLogEntry {
    pub direction: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
}
