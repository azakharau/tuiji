use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::{
    app::key_handlers::KeyHandler,
    ui::{
        components::{
            issue_card::{IssueCardComponent, IssueType, Priority},
            kanban_board::KanbanBoard,
        },
        screens::{Screen, ScreenState},
    },
};

pub struct CurrentSprintScreen<'a> {
    issues: Vec<IssueCardComponent<'a>>,
}

impl<'a> Screen for CurrentSprintScreen<'a> {
    fn draw(&mut self, frame: &mut Frame) {
        let kanban_board = KanbanBoard::new(
            1,
            "Current Sprint".to_string(),
            self.issues.iter().collect(),
        );
        frame.render_widget(kanban_board, frame.area());
    }

    fn name(&self) -> &'static str {
        "Current Sprint"
    }
}

impl KeyHandler for CurrentSprintScreen<'_> {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ScreenState {
        match key_event.code {
            crossterm::event::KeyCode::Char('q') => ScreenState::Quit,
            _ => ScreenState::Stay,
        }
    }
}

impl<'a> Default for CurrentSprintScreen<'a> {
    fn default() -> Self {
        let issues = vec![
            IssueCardComponent {
                key: "SPRINT-123",
                summary: "Implement current sprint screen",
                epic: "",
                status: "In Progress",
                issue_type: IssueType::Story,
                priority: Priority::High,
                assignee: "Alice",
                story_points: Some(5),
            },
            IssueCardComponent {
                key: "SPRINT-124",
                summary: "Fix bug in sprint view",
                epic: "",
                status: "To Do",
                issue_type: IssueType::Bug,
                priority: Priority::Medium,
                assignee: "Bob",
                story_points: Some(3),
            },
            IssueCardComponent {
                key: "SPRINT-125",
                summary: "Write tests for sprint functionality",
                epic: "",
                status: "Done",
                issue_type: IssueType::Task,
                priority: Priority::Low,
                assignee: "Charlie",
                story_points: Some(2),
            },
            IssueCardComponent {
                key: "SPRINT-126",
                summary: "Update documentation for sprint features",
                epic: "",
                status: "In Review",
                issue_type: IssueType::Task,
                priority: Priority::Low,
                assignee: "Dana",
                story_points: Some(1),
            },
            IssueCardComponent {
                key: "SPRINT-127",
                summary: "Refactor sprint management code",
                epic: "",
                status: "To Do",
                issue_type: IssueType::Story,
                priority: Priority::High,
                assignee: "Eve",
                story_points: Some(8),
            },
            IssueCardComponent {
                key: "SPRINT-128",
                summary: "Design new sprint board UI",
                epic: "",
                status: "In Progress",
                issue_type: IssueType::Story,
                priority: Priority::Critical,
                assignee: "Frank",
                story_points: Some(13),
            },
            IssueCardComponent {
                key: "SPRINT-129",
                summary: "Optimize sprint data loading",
                epic: "",
                status: "To Do",
                issue_type: IssueType::Task,
                priority: Priority::Medium,
                assignee: "Grace",
                story_points: Some(5),
            },
            IssueCardComponent {
                key: "SPRINT-130",
                summary: "Conduct sprint retrospective meeting",
                epic: "",
                status: "Done",
                issue_type: IssueType::Task,
                priority: Priority::Low,
                assignee: "Heidi",
                story_points: Some(2),
            },
            IssueCardComponent {
                key: "SPRINT-131",
                summary: "Implement sprint burndown chart",
                epic: "",
                status: "In Review",
                issue_type: IssueType::Story,
                priority: Priority::High,
                assignee: "Ivan",
                story_points: Some(8),
            },
            IssueCardComponent {
                key: "SPRINT-132",
                summary: "Set up sprint notifications",
                epic: "",
                status: "To Do",
                issue_type: IssueType::Task,
                priority: Priority::Medium,
                assignee: "Judy",
                story_points: Some(3),
            },
            IssueCardComponent {
                key: "SPRINT-133",
                summary: "Analyze sprint performance metrics",
                epic: "",
                status: "Code Review",
                issue_type: IssueType::Story,
                priority: Priority::High,
                assignee: "Kevin",
                story_points: Some(5),
            },
            IssueCardComponent {
                key: "SPRINT-134",
                summary: "Integrate sprint tools with CI/CD pipeline",
                epic: "",
                status: "Code Review",
                issue_type: IssueType::Story,
                priority: Priority::Critical,
                assignee: "Laura",
                story_points: Some(13),
            },
        ];

        Self { issues }
    }
}
