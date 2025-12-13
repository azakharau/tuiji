use std::sync::Arc;

use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    app::{
        key_handlers::{ActionHint, KeyHandler},
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
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
    selected_col: usize,
    selected_row: usize,
    scroll_offset: usize,
    rows_visible: usize,
}

impl CurrentSprintScreen {
    pub async fn new(cfg: &AppConfig, mode: Mode) -> Result<Self> {
        let mut isuses = Vec::new();
        let jira = JiraClient::new(
            cfg.jira.base_url.as_str(),
            cfg.jira.username.as_str(),
            cfg.jira.api_token.as_str(),
        )?;
        let board_cfg = jira.get_board_config(BOARD_ID).await?;

        let jira_issues = jira.get_current_sprint_issues(BOARD_ID).await?;

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
impl Screen for CurrentSprintScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let issue_height = self.issues.first().map(|i| i.height()).unwrap_or(8);
        let bottom_bar = BottomBar::new(self.mode.to_owned(), self.actions.clone());
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
        self.rows_visible = ((layout[1].height.saturating_sub(1)) / issue_height.max(1)).max(1) as usize;
        self.clamp_selection();
        self.ensure_selection_visible();
        let kanban_board = KanbanBoard::new(
            1,
            "Current Sprint".to_string(),
            self.issues.clone(),
            &self.board_cfg,
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
}

impl KeyHandler for CurrentSprintScreen {
    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        bindings: &crate::config::KeyBindings,
    ) -> ScreenState {
        use crate::app::key_handlers::binding_matches;
        if binding_matches(&key_event, &bindings.refresh) {
            return ScreenState::Refresh;
        }
        match key_event.code {
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                self.move_down();
                return ScreenState::Refresh;
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                self.move_up();
                return ScreenState::Refresh;
            }
            crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                self.move_left();
                return ScreenState::Refresh;
            }
            crossterm::event::KeyCode::Char('l') | crossterm::event::KeyCode::Right => {
                self.move_right();
                return ScreenState::Refresh;
            }
            _ => {}
        }
        ScreenState::Stay
    }
}

impl CurrentSprintScreen {
    pub fn action_hints(bindings: &crate::config::KeyBindings) -> Arc<Vec<ActionHint>> {
        Arc::new(vec![
            ActionHint {
                binding: format!("{}/↑", bindings.previous),
                description: "Up".to_string(),
            },
            ActionHint {
                binding: format!("{}/↓", bindings.next),
                description: "Down".to_string(),
            },
            ActionHint {
                binding: "h/←".to_string(),
                description: "Prev column".to_string(),
            },
            ActionHint {
                binding: "l/→".to_string(),
                description: "Next column".to_string(),
            },
            ActionHint {
                binding: bindings.refresh.clone(),
                description: "Refresh".to_string(),
            },
            ActionHint {
                binding: bindings.quit.clone(),
                description: "Quit".to_string(),
            },
        ])
    }

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

    fn move_down(&mut self) {
        let counts = self.column_counts();
        let count = counts.get(self.selected_col).copied().unwrap_or(0);
        if self.selected_row + 1 < count {
            self.selected_row += 1;
        }
        self.ensure_selection_visible();
    }

    fn move_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
        self.ensure_selection_visible();
    }

    fn move_left(&mut self) {
        if self.board_cfg.columns.is_empty() {
            return;
        }
        let target = if self.selected_col == 0 {
            self.board_cfg.columns.len() - 1
        } else {
            self.selected_col - 1
        };
        self.selected_col = self.find_non_empty_from(target, -1);
        self.clamp_selection();
        self.ensure_selection_visible();
    }

    fn move_right(&mut self) {
        if self.board_cfg.columns.is_empty() {
            return;
        }
        let target = (self.selected_col + 1) % self.board_cfg.columns.len();
        self.selected_col = self.find_non_empty_from(target, 1);
        self.clamp_selection();
        self.ensure_selection_visible();
    }

    fn find_non_empty_from(&self, start: usize, dir: i32) -> usize {
        let counts = self.column_counts();
        if counts.iter().all(|&c| c == 0) {
            return 0;
        }
        let len = counts.len();
        let mut idx = start % len;
        for _ in 0..len {
            if counts[idx] > 0 {
                return idx;
            }
            idx = (((idx as i32 + dir).rem_euclid(len as i32)) as usize) % len;
        }
        start
    }
}
