use std::sync::Arc;

use crate::{client::jira::BoardConfig, ui::components::issue_card::IssueCardComponent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SprintViewMode {
    Kanban,
    Table,
}

pub struct CurrentSprintState {
    issues: Arc<Vec<IssueCardComponent>>,
    board_cfg: BoardConfig,
    selected_col: usize,
    selected_row: usize,
    scroll_offset: usize,
    rows_visible: usize,
    view_mode: SprintViewMode,
}

impl CurrentSprintState {
    pub fn new(issues: Vec<IssueCardComponent>, board_cfg: BoardConfig) -> Self {
        Self {
            issues: Arc::new(issues),
            board_cfg,
            selected_col: 0,
            selected_row: 0,
            scroll_offset: 0,
            rows_visible: 1,
            view_mode: SprintViewMode::Kanban,
        }
    }

    pub fn issues(&self) -> &Arc<Vec<IssueCardComponent>> {
        &self.issues
    }

    pub fn board_cfg(&self) -> &BoardConfig {
        &self.board_cfg
    }

    pub fn selected_col(&self) -> usize {
        self.selected_col
    }

    pub fn selected_row(&self) -> usize {
        self.selected_row
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn rows_visible(&self) -> usize {
        self.rows_visible
    }

    pub fn view_mode(&self) -> SprintViewMode {
        self.view_mode
    }

    pub fn set_rows_visible(&mut self, rows_visible: usize) {
        self.rows_visible = rows_visible.max(1);
    }

    pub fn set_selected_col(&mut self, value: usize) {
        self.selected_col = value;
    }

    pub fn set_selected_row(&mut self, value: usize) {
        self.selected_row = value;
    }

    pub fn set_scroll_offset(&mut self, value: usize) {
        self.scroll_offset = value;
    }
}
