use std::sync::Arc;

use color_eyre::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::Block,
};

use crate::{
    app::{
        key_handlers::{ActionHint, ActionId, Command, KeyHandler},
        state::Mode,
    },
    client::jira::BoardConfig,
    data::AppRepository,
    ui::{
        components::{
            bottom_bar::BottomBar,
            issue_card::{IssueCardComponent, IssueType, Priority},
            kanban_board::KanbanBoard,
        },
        context::RenderContext,
        screens::{Screen, ScreenState},
        screens::CommandLineCommand,
    },
};
pub struct CurrentKanbanSprintScreen {
    issues: Arc<Vec<IssueCardComponent>>,
    board_cfg: BoardConfig,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
    selected_col: usize,
    selected_row: usize,
    scroll_offset: usize,
    rows_visible: usize,
}

impl CurrentKanbanSprintScreen {
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let mut isuses = Vec::new();
        let board_cfg = repo.board_config(board_id).await?;
        let jira_issues = repo.current_sprint_issues(board_id).await?;

        jira_issues.into_iter().for_each(|issue| {
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

            isuses.push(issue_card);
        });
        Ok(Self {
            issues: Arc::new(isuses),
            board_cfg,
            actions: Arc::new(vec![]),
            mode,
            selected_col: 0,
            selected_row: 0,
            scroll_offset: 0,
            rows_visible: 1,
        })
    }
}
impl Screen for CurrentKanbanSprintScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        let base_style = ratatui::style::Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());
        self.ensure_valid_column();
        let issue_height = self.issues.first().map(|i| i.height()).unwrap_or(8);
        let bottom_bar = BottomBar::new(self.mode.to_owned(), self.actions.clone(), context);
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
        self.rows_visible =
            ((layout[1].height.saturating_sub(1)) / issue_height.max(1)).max(1) as usize;
        self.clamp_selection();
        self.ensure_selection_visible();
        let kanban_board = KanbanBoard::new(
            1,
            "Current Sprint".to_string(),
            self.issues.clone(),
            &self.board_cfg,
            context,
            self.selected_col,
            self.selected_row,
            self.scroll_offset,
            self.rows_visible,
        );
        frame.render_widget(kanban_board, layout[1]);
        frame.render_widget(bottom_bar, layout[2]);
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

impl KeyHandler for CurrentKanbanSprintScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::Refresh => ScreenState::Refresh,
            ActionId::MoveDown => {
                self.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                self.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveLeft => {
                self.move_left(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveRight => {
                self.move_right(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                self.go_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                self.go_bottom();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}

impl CurrentKanbanSprintScreen {
    fn column_counts(&self) -> Vec<usize> {
        let mut counts = vec![0; self.board_cfg.columns.len()];
        for issue in self.issues.iter() {
            if let Some(idx) = self
                .board_cfg
                .columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(&issue.status))
            {
                counts[idx] += 1;
            }
        }
        counts
    }

    fn ensure_valid_column(&mut self) {
        let len = self.board_cfg.columns.len();
        if len == 0 {
            self.selected_col = 0;
            self.selected_row = 0;
            self.scroll_offset = 0;
            return;
        }
        if self.selected_col >= len {
            self.selected_col = len - 1;
        }
        let counts = self.column_counts();
        if counts.get(self.selected_col).copied().unwrap_or(0) > 0 {
            return;
        }
        // prefer nearest non-empty to the left, then right
        if let Some(left) = (0..=self.selected_col)
            .rev()
            .find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            self.selected_col = left;
        } else if let Some(right) =
            (self.selected_col + 1..len).find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            self.selected_col = right;
        } else {
            // all empty
            self.selected_col = 0;
            self.selected_row = 0;
            self.scroll_offset = 0;
        }
    }

    fn clamp_selection(&mut self) {
        let counts = self.column_counts();
        let col_count = counts.get(self.selected_col).copied().unwrap_or(0);
        if col_count == 0 {
            self.selected_row = 0;
        } else if self.selected_row >= col_count {
            self.selected_row = col_count - 1;
        }
    }

    fn ensure_selection_visible(&mut self) {
        let counts = self.column_counts();
        let col_count = counts.get(self.selected_col).copied().unwrap_or(0);
        let rows_visible = self.rows_visible.max(1);
        // Keep a single scroll_offset applied to all columns; clamp to available rows in current column.
        if col_count <= rows_visible {
            self.scroll_offset = 0;
            return;
        }
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        } else if self.selected_row >= self.scroll_offset + rows_visible {
            self.scroll_offset = self.selected_row + 1 - rows_visible;
        }
        let max_offset = counts
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .saturating_sub(rows_visible);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    fn move_down(&mut self, n: usize) {
        let counts = self.column_counts();
        let count = counts.get(self.selected_col).copied().unwrap_or(0);
        if count == 0 {
            return;
        }
        let max_row = count - 1;
        self.selected_row = (self.selected_row + n).min(max_row);
        self.ensure_selection_visible();
    }

    fn move_up(&mut self, n: usize) {
        if self.selected_row > 0 {
            self.selected_row = self.selected_row.saturating_sub(n);
        }
        self.ensure_selection_visible();
    }

    fn move_left(&mut self, steps: usize) {
        if self.board_cfg.columns.is_empty() {
            return;
        }
        let counts = self.column_counts();
        let target = self.selected_col.saturating_sub(steps);
        if counts.get(target).copied().unwrap_or(0) > 0 {
            self.selected_col = target;
        } else if target > 0
            && let Some(idx) = (0..target)
                .rev()
                .find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            self.selected_col = idx;
        }
        self.clamp_selection();
        self.ensure_selection_visible();
    }

    fn move_right(&mut self, steps: usize) {
        if self.board_cfg.columns.is_empty() {
            return;
        }
        let counts = self.column_counts();
        let len = self.board_cfg.columns.len();
        let target = (self.selected_col + steps).min(len.saturating_sub(1));
        if counts.get(target).copied().unwrap_or(0) > 0 {
            self.selected_col = target;
        } else if target + 1 < len
            && let Some(idx) = (target + 1..len).find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            self.selected_col = idx;
        }
        self.clamp_selection();
        self.ensure_selection_visible();
    }

    fn go_top(&mut self) {
        self.selected_row = 0;
        self.scroll_offset = 0;
    }

    fn go_bottom(&mut self) {
        let counts = self.column_counts();
        if let Some(max_rows) = counts.get(self.selected_col)
            && *max_rows > 0
        {
            self.selected_row = *max_rows - 1;
            self.ensure_selection_visible();
        }
    }
}
