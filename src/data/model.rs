use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Default)]
pub struct BoardConfig {
    pub columns: Vec<BoardColumn>,
    pub estimation: Estimation,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Estimation {
    StoryPoints(String),
    DateBased(String),
}

impl Default for Estimation {
    fn default() -> Self {
        Self::StoryPoints("customfield_10002".to_string())
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct BoardColumn {
    pub name: String,
    pub statuses: Vec<ColumnStatusRef>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ColumnStatusRef {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct IssueDraft {
    pub project_key: String,
    pub issue_type: String,
    pub summary: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IssuePatch {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
}

impl IssuePatch {
    pub fn is_empty(&self) -> bool {
        self.summary.is_none() && self.description.is_none() && self.priority.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionChoice {
    pub id: String,
    pub name: String,
    pub to_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueMutation {
    Create(IssueDraft),
    Patch {
        key: String,
        patch: IssuePatch,
    },
    Comment {
        key: String,
        body: String,
    },
    Transition {
        key: String,
        id: String,
        to_status: String,
    },
    AssignToMe {
        key: String,
    },
}

/// Payload stored in `outbox.change_set`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum OutboxChange {
    Fields {
        fields: serde_json::Map<String, serde_json::Value>,
    },
    Transition {
        id: String,
        to_status: String,
    },
    /// `issue_key` is required: `outbox.entity_id` holds the local comment id,
    /// not the issue key, so the push would otherwise have no target.
    Comment {
        issue_key: String,
        body: String,
    },
    Assignee {
        account_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn outbox_change_should_round_trip_every_variant() {
        let changes = [
            OutboxChange::Fields {
                fields: serde_json::Map::from_iter([
                    ("summary".to_string(), json!("Updated summary")),
                    ("priority".to_string(), json!({ "name": "High" })),
                ]),
            },
            OutboxChange::Transition {
                id: "31".to_string(),
                to_status: "Done".to_string(),
            },
            OutboxChange::Comment {
                issue_key: "TUIJI-42".to_string(),
                body: "Queued comment".to_string(),
            },
            OutboxChange::Assignee {
                account_id: "account-123".to_string(),
            },
        ];

        for change in changes {
            let serialized = serde_json::to_string(&change).unwrap();
            let deserialized: OutboxChange = serde_json::from_str(&serialized).unwrap();

            assert_eq!(deserialized, change);
        }
    }

    #[test]
    fn outbox_change_should_serialize_issue_key_under_a_stable_name() {
        let change = OutboxChange::Comment {
            issue_key: "TUIJI-42".to_string(),
            body: "Queued comment".to_string(),
        };

        let serialized = serde_json::to_value(change).unwrap();

        assert_eq!(serialized["op"], "comment");
        assert_eq!(serialized["issue_key"], "TUIJI-42");
    }

    #[test]
    fn outbox_change_should_deserialize_the_v0_1_0_on_disk_shape() {
        let persisted = r#"{"op":"comment","issue_key":"TUIJI-42","body":"Queued comment"}"#;

        let change: OutboxChange = serde_json::from_str(persisted).unwrap();

        match change {
            OutboxChange::Comment { issue_key, body } => {
                assert_eq!(issue_key, "TUIJI-42");
                assert_eq!(body, "Queued comment");
            }
            other => panic!("expected persisted comment change, got {other:?}"),
        }
    }
}
