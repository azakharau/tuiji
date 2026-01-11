mod controller;
mod kanban;
mod state;
mod table;
mod view;

use std::sync::Arc;

use color_eyre::Result;
use ratatui::Frame;

use crate::{
    app::{
        key_handlers::{ActionHint, Command, KeyHandler},
        state::Mode,
    },
    data::AppRepository,
    ui::{
        components::issue_card::{IssueCardComponent, IssueType, Priority},
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::CurrentSprintController;
use state::CurrentSprintState;
use view::CurrentSprintView;

pub struct CurrentSprintScreen {
    state: CurrentSprintState,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl CurrentSprintScreen {
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let board_cfg = repo.board_config(board_id).await?;
        let jira_issues = repo.current_sprint_issues(board_id).await?;

        let mut issues = Vec::with_capacity(jira_issues.len());
        for issue in jira_issues {
            let issue_type = IssueType::from(issue.issue_type.as_str());
            let priority = Priority::from(issue.priority.as_str());
            let issue_card = IssueCardComponent {
                key: issue.key,
                summary: issue.summary,
                epic: issue.epic,
                status: issue.status,
                issue_type,
                priority,
                assignee: issue.assignee,
                story_points: issue.story_points,
            };
            issues.push(issue_card);
        }
        Ok(Self {
            state: CurrentSprintState::new(issues, board_cfg),
            actions: Arc::new(vec![]),
            mode,
        })
    }

    fn issue_height(&self) -> u16 {
        self.state.issues().first().map(|i| i.height()).unwrap_or(8)
    }
}

impl Screen for CurrentSprintScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        let issue_height = self.issue_height();
        let layout = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(2),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(frame.area());
        let rows_visible =
            ((layout[1].height.saturating_sub(1)) / issue_height.max(1)).max(1) as usize;
        CurrentSprintController::update_rows_visible(&mut self.state, rows_visible);
        CurrentSprintView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Current Sprint"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => ScreenState::Stay,
            CommandLineCommand::WriteQuit => ScreenState::Close,
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

impl KeyHandler for CurrentSprintScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        CurrentSprintController::handle_command(&mut self.state, command)
    }
}
