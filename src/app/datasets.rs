use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCategory {
    pub id: u32,
    pub key: String,
    pub name: String,
    pub color_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumn {
    pub id: String,
    pub name: String,
    pub status_ids: Vec<String>,
    pub status_category_key: String,
    pub wip_limit: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintMetadata {
    pub id: u32,
    pub name: String,
    pub goal: String,
    pub state: String,
    pub start_date: String,
    pub end_date: String,
    pub complete_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueType {
    pub id: String,
    pub name: String,
    pub icon_url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: u32,
    pub key: String,
    pub summary: String,
    pub status_id: String,
    pub issue_type_id: String,
    pub assignee: String,
    pub priority: String,
    pub story_points: Option<f32>,
    pub labels: Vec<String>,
    pub flagged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentSprintDataset {
    pub board_id: u32,
    pub board_name: String,
    pub board_type: String,
    pub sprint: SprintMetadata,
    pub status_categories: Vec<StatusCategory>,
    pub statuses: Vec<JiraStatus>,
    pub columns: Vec<BoardColumn>,
    pub issue_types: Vec<IssueType>,
    pub issues: Vec<JiraIssue>,
}

pub fn current_sprint_sample() -> CurrentSprintDataset {
    CurrentSprintDataset {
        board_id: 112,
        board_name: "Mobile Platform".into(),
        board_type: "scrum".into(),
        sprint: SprintMetadata {
            id: 230,
            name: "Sprint 54 - Solstice".into(),
            goal: "Ship the refreshed onboarding funnel and stabilize biometric auth.".into(),
            state: "active".into(),
            start_date: "2025-12-02T09:00:00.000-08:00".into(),
            end_date: "2025-12-16T17:00:00.000-08:00".into(),
            complete_by: "2025-12-16T17:30:00.000-08:00".into(),
        },
        status_categories: vec![
            StatusCategory {
                id: 1,
                key: "new".into(),
                name: "To Do".into(),
                color_name: "blue-gray".into(),
            },
            StatusCategory {
                id: 2,
                key: "in-flight".into(),
                name: "In Progress".into(),
                color_name: "yellow".into(),
            },
            StatusCategory {
                id: 3,
                key: "done".into(),
                name: "Done".into(),
                color_name: "green".into(),
            },
        ],
        statuses: vec![
            JiraStatus {
                id: "10001".into(),
                name: "To Do".into(),
                description: "Items that passed refinement and await development.".into(),
                category_key: "new".into(),
            },
            JiraStatus {
                id: "10002".into(),
                name: "In Progress".into(),
                description: "Work that is actively being implemented.".into(),
                category_key: "in-flight".into(),
            },
            JiraStatus {
                id: "10800".into(),
                name: "Code Review".into(),
                description: "Changes waiting for peer review or automated checks.".into(),
                category_key: "in-flight".into(),
            },
            JiraStatus {
                id: "10003".into(),
                name: "Done".into(),
                description: "Completed work that satisfies the Definition of Done.".into(),
                category_key: "done".into(),
            },
        ],
        columns: vec![
            BoardColumn {
                id: "col-todo".into(),
                name: "To Do".into(),
                status_ids: vec!["10001".into()],
                status_category_key: "new".into(),
                wip_limit: None,
            },
            BoardColumn {
                id: "col-in-progress".into(),
                name: "In Progress".into(),
                status_ids: vec!["10002".into()],
                status_category_key: "in-flight".into(),
                wip_limit: Some(5),
            },
            BoardColumn {
                id: "col-code-review".into(),
                name: "Code Review".into(),
                status_ids: vec!["10800".into()],
                status_category_key: "in-flight".into(),
                wip_limit: Some(4),
            },
            BoardColumn {
                id: "col-done".into(),
                name: "Done".into(),
                status_ids: vec!["10003".into()],
                status_category_key: "done".into(),
                wip_limit: None,
            },
        ],
        issue_types: vec![
            IssueType {
                id: "10000".into(),
                name: "Story".into(),
                icon_url: "https://cdn.atlassian.com/jira-core-icons/story.svg".into(),
                description: "A user-facing increment of value delivered within a sprint.".into(),
            },
            IssueType {
                id: "10001".into(),
                name: "Task".into(),
                icon_url: "https://cdn.atlassian.com/jira-core-icons/task.svg".into(),
                description: "Team-managed work that supports delivery or maintenance.".into(),
            },
            IssueType {
                id: "10002".into(),
                name: "Bug".into(),
                icon_url: "https://cdn.atlassian.com/jira-core-icons/bug.svg".into(),
                description: "A defect that prevents the product from working as expected.".into(),
            },
        ],
        issues: vec![
            JiraIssue {
                id: 50100,
                key: "MOB-1421".into(),
                summary: "Story: Sign-in session refresh".into(),
                status_id: "10001".into(),
                issue_type_id: "10000".into(),
                assignee: "Irina Petrov".into(),
                priority: "Medium".into(),
                story_points: Some(5.0),
                labels: vec!["mobile".into(), "auth".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50101,
                key: "MOB-1428".into(),
                summary: "Story: Offline cache warm-up".into(),
                status_id: "10002".into(),
                issue_type_id: "10000".into(),
                assignee: "Akira Sato".into(),
                priority: "High".into(),
                story_points: Some(8.0),
                labels: vec!["offline".into(), "sync".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50102,
                key: "MOB-1430".into(),
                summary: "Task: Instrument crash-free metric".into(),
                status_id: "10002".into(),
                issue_type_id: "10001".into(),
                assignee: "Diego Martinez".into(),
                priority: "Medium".into(),
                story_points: Some(3.0),
                labels: vec!["observability".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50103,
                key: "MOB-1434".into(),
                summary: "Bug: App freeze on biometric enable".into(),
                status_id: "10800".into(),
                issue_type_id: "10002".into(),
                assignee: "Sara Chen".into(),
                priority: "Critical".into(),
                story_points: None,
                labels: vec!["bug".into(), "ios".into()],
                flagged: true,
            },
            JiraIssue {
                id: 50104,
                key: "MOB-1437".into(),
                summary: "Story: Adaptive typography scale".into(),
                status_id: "10800".into(),
                issue_type_id: "10000".into(),
                assignee: "Dev Patel".into(),
                priority: "Low".into(),
                story_points: Some(2.0),
                labels: vec!["ux".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50105,
                key: "MOB-1440".into(),
                summary: "Task: Update OWASP dependency baseline".into(),
                status_id: "10003".into(),
                issue_type_id: "10001".into(),
                assignee: "Mia Novak".into(),
                priority: "Medium".into(),
                story_points: Some(1.0),
                labels: vec!["security".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50106,
                key: "MOB-1442".into(),
                summary: "Bug: Android push token rotation".into(),
                status_id: "10003".into(),
                issue_type_id: "10002".into(),
                assignee: "Kwame Mensah".into(),
                priority: "High".into(),
                story_points: None,
                labels: vec!["android".into(), "push".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50107,
                key: "MOB-1445".into(),
                summary: "Story: VoiceOver discoverability".into(),
                status_id: "10001".into(),
                issue_type_id: "10000".into(),
                assignee: "Maya Kapoor".into(),
                priority: "Medium".into(),
                story_points: Some(5.0),
                labels: vec!["accessibility".into()],
                flagged: false,
            },
            JiraIssue {
                id: 50108,
                key: "MOB-1448".into(),
                summary: "Bug: Regression in dark theme palette".into(),
                status_id: "10800".into(),
                issue_type_id: "10002".into(),
                assignee: "Liam O'Connor".into(),
                priority: "High".into(),
                story_points: None,
                labels: vec!["ui".into(), "theme".into()],
                flagged: true,
            },
        ],
    }
}
