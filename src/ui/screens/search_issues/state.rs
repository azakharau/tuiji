use ratatui::widgets::Row;

use crate::data::IssueSummary;
use crate::ui::components::list::TableRow;

pub struct SearchIssuesState {
    rows: Vec<TableRow<'static>>,
    selected_index: usize,
}

impl SearchIssuesState {
    pub fn new(issues: Vec<IssueSummary>) -> Self {
        Self {
            rows: build_rows(issues),
            selected_index: 0,
        }
    }

    pub fn rows(&self) -> &[TableRow<'static>] {
        &self.rows
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn move_up(&mut self, n: usize) {
        if self.rows.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = self.selected_index.saturating_sub(step);
    }

    pub fn move_down(&mut self, n: usize) {
        if self.rows.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = (self.selected_index + step).min(self.rows.len() - 1);
    }

    pub fn move_top(&mut self) {
        if !self.rows.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn move_bottom(&mut self) {
        if !self.rows.is_empty() {
            self.selected_index = self.rows.len() - 1;
        }
    }
}

fn build_rows(issues: Vec<IssueSummary>) -> Vec<TableRow<'static>> {
    let mut rows = Vec::with_capacity(issues.len());
    for issue in issues {
        let IssueSummary {
            key,
            summary,
            status,
            ..
        } = issue;
        rows.push(Row::new(vec![key, summary, status]));
    }
    rows
}
