use std::sync::Arc;

use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, ScrollbarState, TableState},
};

use crate::{
    app::{
        key_handlers::{ActionHint, Command, KeyHandler},
        state::Mode,
    },
    client::jira::BoardConfig,
    data::AppRepository,
    ui::{
        components::bottom_bar::BottomBar,
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

const _SELECTED_ROW_COLOR: Color = Color::Cyan;
#[derive(Debug, Clone)]
pub enum Status {
    ToDo,
    InProgress,
    CodeReview,
    Done,
}

impl From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "To Do" => Status::ToDo,
            "In Progress" => Status::InProgress,
            "Code Review" => Status::CodeReview,
            "Done" => Status::Done,
            _ => Status::ToDo,
        }
    }
}

impl From<Status> for &str {
    fn from(status: Status) -> Self {
        match status {
            Status::ToDo => "To Do",
            Status::InProgress => "In Progress",
            Status::CodeReview => "Code Review",
            Status::Done => "Done",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Priority {
    NoBusinessValue,
    Low,
    Lowest,
    #[default]
    Medium,
    High,
    Critical,
}

impl From<&str> for Priority {
    fn from(s: &str) -> Self {
        match s {
            "No Business Value" => Priority::NoBusinessValue,
            "Low" => Priority::Low,
            "Lowest" => Priority::Lowest,
            "Medium" => Priority::Medium,
            "High" => Priority::High,
            "Critical" => Priority::Critical,
            _ => Priority::Medium,
        }
    }
}

impl Priority {
    pub fn as_span(&'_ self) -> Span<'_> {
        match self {
            Priority::NoBusinessValue => {
                Span::styled("No Business Value", Style::default().fg(Color::Gray))
            }
            Priority::Low => Span::styled("Low", Style::default().fg(Color::Green)),
            Priority::Lowest => Span::styled("Lowest", Style::default().fg(Color::Cyan)),
            Priority::Medium => Span::styled("Medium", Style::default().fg(Color::Yellow)),
            Priority::High => Span::styled("High", Style::default().fg(Color::Red)),
            Priority::Critical => Span::styled("Critical", Style::default().fg(Color::Magenta)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum IssueType {
    Bug,
    #[default]
    Task,
    Story,
    Subtask,
}

impl From<&str> for IssueType {
    fn from(s: &str) -> Self {
        match s {
            "Bug" => IssueType::Bug,
            "Task" => IssueType::Task,
            "Story" => IssueType::Story,
            "Subtask" => IssueType::Subtask,
            _ => IssueType::Task,
        }
    }
}

impl IssueType {
    pub fn as_span(&'_ self) -> Span<'_> {
        match self {
            IssueType::Bug => Span::styled("B", Style::default().fg(Color::Red)),
            IssueType::Task => Span::styled("T", Style::default().fg(Color::Blue)),
            IssueType::Story => Span::styled("S", Style::default().fg(Color::Green)),
            IssueType::Subtask => Span::styled("ST", Style::default().fg(Color::Cyan)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
    pub epic: Option<String>,
    pub status: String,
    pub issue_type: IssueType,
    pub assignee: String,
    pub priority: Priority,
    pub story_points: Option<f64>,
}

pub struct CurrentSprintTableScreen {
    _issues: Arc<Vec<JiraIssue>>,
    _board_cfg: BoardConfig,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
    _state: TableState,
    _scroll_state: ScrollbarState,
}

impl CurrentSprintTableScreen {
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let board_cfg = repo.board_config(board_id).await?;
        let jira_issues = repo.current_sprint_issues(board_id).await?;
        let mut issues = Vec::with_capacity(jira_issues.len());
        for issue in jira_issues {
            issues.push(JiraIssue {
                key: issue.key,
                summary: issue.summary,
                epic: issue.epic,
                status: issue.status,
                issue_type: IssueType::from(issue.issue_type.as_str()),
                assignee: issue.assignee,
                priority: Priority::from(issue.priority.as_str()),
                story_points: issue.story_points,
            });
        }

        Ok(Self {
            _issues: Arc::new(issues),
            _board_cfg: board_cfg,
            actions: Arc::new(Vec::new()),
            mode,
            _state: TableState::default(),
            _scroll_state: ScrollbarState::default(),
        })
    }
}

impl Screen for CurrentSprintTableScreen {
    fn draw(&mut self, _frame: &mut Frame, _context: &RenderContext) {
        let base_style = Style::default()
            .fg(_context.colors().text)
            .bg(_context.colors().background);
        let main_frame = Block::default().title(self.name()).style(base_style);
        let inner_area = main_frame.inner(_frame.area());
        let bottom_bar = BottomBar::new(self.mode.to_owned(), self.actions.clone());

        let [_body, bottom] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner_area);

        _frame.render_widget(main_frame, _frame.area());
        _frame.render_widget(bottom_bar, bottom);
    }

    fn name(&self) -> &'static str {
        "Sprint Issues"
    }

    fn set_action_hints(&mut self, _actions: Arc<Vec<ActionHint>>) {
        self.actions = _actions;
    }

    fn set_mode(&mut self, _mode: Mode) {
        self.mode = _mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => ScreenState::Stay,
            CommandLineCommand::WriteQuit => ScreenState::Close,
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

impl KeyHandler for CurrentSprintTableScreen {
    fn handle_command(&mut self, _command: Command) -> ScreenState {
        ScreenState::Stay
    }
}
