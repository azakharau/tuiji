use std::sync::Arc;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    app::{
        key_handlers::{ActionItem, KeyHandler},
        state::Mode,
    },
    client::jira::{BoardConfig, JiraClient},
    config::AppConfig,
    ui::{
        components::{
            bottom_bar::BottomBar,
            issue_card::{IssueCardComponent, IssueType, Priority},
            kanban_board::KanbanBoard,
        },
        screens::{Screen, ScreenState},
    },
};
const BOARD_ID: u64 = 175;

pub struct CurrentSprintScreen {
    issues: Arc<Vec<IssueCardComponent>>,
    board_cfg: BoardConfig,
    actions: Arc<Vec<ActionItem>>,
    mode: Mode,
}

impl CurrentSprintScreen {
    pub fn new(cfg: &AppConfig, mode: Mode) -> Self {
        let mut isuses = Vec::new();
        let jira = JiraClient::new(
            cfg.jira.base_url.as_str(),
            cfg.jira.username.as_str(),
            cfg.jira.api_token.as_str(),
        );
        let board_cfg = jira
            .get_board_config(BOARD_ID)
            .expect("Failed to fetch board config");

        let jira_issues = jira
            .get_current_sprint_issues(BOARD_ID)
            .expect("Failed to fetch current sprint issues");

        jira_issues.into_iter().for_each(|issue| {
            let key = issue.key.to_string();
            let summary = issue.summary().unwrap_or_default();
            let epic = None;
            let status = match issue.status() {
                Some(st) => st.name.to_uppercase(),
                None => "TODO".to_string(),
            };
            let issue_type = match issue.issue_type() {
                Some(it) => IssueType::from(it.name.as_str()),
                None => IssueType::default(),
            };
            let assignee = match issue.assignee() {
                Some(user) => user.display_name,
                None => "Unassigned".to_string(),
            };
            let priority = match issue.priority() {
                Some(pr) => Priority::from(pr.name.as_str()),
                None => Priority::default(),
            };
            let story_points = board_cfg.estimation.extract_value(&issue);

            let issue_card = IssueCardComponent {
                key,
                summary,
                epic,
                status,
                issue_type,
                priority,
                assignee,
                story_points,
            };

            isuses.push(issue_card);
        });
        Self {
            issues: Arc::new(isuses),
            board_cfg,
            actions: Arc::new(vec![]),
            mode,
        }
    }
}
impl Screen for CurrentSprintScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let kanban_board = KanbanBoard::new(
            1,
            "Current Sprint".to_string(),
            self.issues.clone(),
            &self.board_cfg,
        );
        let bottom_bar = BottomBar::new(self.mode.to_owned(), self.actions.clone());
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
        frame.render_widget(kanban_board, layout[1]);
        frame.render_widget(bottom_bar, layout[2]);
    }

    fn name(&self) -> &'static str {
        "Current Sprint"
    }
}

impl KeyHandler for CurrentSprintScreen {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ScreenState {
        match key_event.code {
            crossterm::event::KeyCode::Char('q') => ScreenState::Quit,
            _ => ScreenState::Stay,
        }
    }
}

// impl Default for CurrentSprintScreen {
//     fn default() -> Self {
//         let issues = vec![
//             IssueCardComponent {
//
//                 key: "SPRINT-123".to_string(),
//                 summary: "Implement current sprint screen".to_string(),
//                 epic: None,
//                 status: "In Progress".to_string(),
//                 issue_type: IssueType::Story,
//                 priority: Priority::High,
//                 assignee: "Alice".to_string(),
//                 story_points: Some(5.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-124".to_string(),
//                 summary: "Fix bug in sprint view".to_string(),
//                 epic: None,
//                 status: "To Do".to_string(),
//                 issue_type: IssueType::Bug,
//                 priority: Priority::Medium,
//                 assignee: "Bob".to_string(),
//                 story_points: Some(3.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-125".to_string(),
//                 summary: "Write tests for sprint functionality".to_string(),
//                 epic: None,
//                 status: "Done".to_string(),
//                 issue_type: IssueType::Task,
//                 priority: Priority::Low,
//                 assignee: "Charlie".to_string(),
//                 story_points: Some(2.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-126".to_string(),
//                 summary: "Update documentation for sprint features".to_string(),
//                 epic: None,
//                 status: "In Review".to_string(),
//                 issue_type: IssueType::Task,
//                 priority: Priority::Low,
//                 assignee: "Dana".to_string(),
//                 story_points: Some(1.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-127".to_string(),
//                 summary: "Refactor sprint management code".to_string(),
//                 epic: None,
//                 status: "To Do".to_string(),
//                 issue_type: IssueType::Story,
//                 priority: Priority::High,
//                 assignee: "Eve".to_string(),
//                 story_points: Some(8.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-128".to_string(),
//                 summary: "Design new sprint board UI".to_string(),
//                 epic: None,
//                 status: "In Progress".to_string(),
//                 issue_type: IssueType::Story,
//                 priority: Priority::Critical,
//                 assignee: "Frank".to_string(),
//                 story_points: Some(13.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-129".to_string(),
//                 summary: "Optimize sprint data loading".to_string(),
//                 epic: None,
//                 status: "To Do".to_string(),
//                 issue_type: IssueType::Task,
//                 priority: Priority::Medium,
//                 assignee: "Grace".to_string(),
//                 story_points: Some(5.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-130".to_string(),
//                 summary: "Conduct sprint retrospective meeting".to_string(),
//                 epic: None,
//                 status: "Done".to_string(),
//                 issue_type: IssueType::Task,
//                 priority: Priority::Low,
//                 assignee: "Heidi".to_string(),
//                 story_points: Some(2.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-131".to_string(),
//                 summary: "Implement sprint burndown chart".to_string(),
//                 epic: None,
//                 status: "In Review".to_string(),
//                 issue_type: IssueType::Story,
//                 priority: Priority::High,
//                 assignee: "Ivan".to_string(),
//                 story_points: Some(8.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-132".to_string(),
//                 summary: "Set up sprint notifications".to_string(),
//                 epic: None,
//                 status: "To Do".to_string(),
//                 issue_type: IssueType::Task,
//                 priority: Priority::Medium,
//                 assignee: "Judy".to_string(),
//                 story_points: Some(3.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-133".to_string(),
//                 summary: "Analyze sprint performance metrics".to_string(),
//                 epic: None,
//                 status: "Code Review".to_string(),
//                 issue_type: IssueType::Story,
//                 priority: Priority::High,
//                 assignee: "Kevin".to_string(),
//                 story_points: Some(5.0),
//             },
//             IssueCardComponent {
//                 key: "SPRINT-134".to_string(),
//                 summary: "Integrate sprint tools with CI/CD pipeline".to_string(),
//                 epic: None,
//                 status: "Code Review".to_string(),
//                 issue_type: IssueType::Story,
//                 priority: Priority::Critical,
//                 assignee: "Laura".to_string(),
//                 story_points: Some(13.0),
//             },
//         ];
//
//         Self {
//             issues: Rc::new(issues),
//             board_cfg: BoardConfig::default(),
//         }
//     }
// }
