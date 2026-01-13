use ratatui::widgets::Row;

use crate::data::IssueSummary;
use crate::ui::components::list::TableRow;

pub struct SearchIssuesState {
    rows: Vec<TableRow<'static>>,
    issue_keys: Vec<String>,
    selected_index: usize,
}

impl SearchIssuesState {
    pub fn new(issues: Vec<IssueSummary>) -> Self {
        let (rows, keys) = build_rows_and_keys(issues);
        Self {
            rows,
            issue_keys: keys,
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

    /// Get the key of the currently selected issue, if any.
    pub fn selected_issue_key(&self) -> Option<&str> {
        self.issue_keys.get(self.selected_index).map(|s| s.as_str())
    }
}

fn build_rows_and_keys(issues: Vec<IssueSummary>) -> (Vec<TableRow<'static>>, Vec<String>) {
    let mut rows = Vec::with_capacity(issues.len());
    let mut keys = Vec::with_capacity(issues.len());
    for issue in issues {
        let IssueSummary {
            key,
            summary,
            status,
            ..
        } = issue;
        keys.push(key.clone());
        rows.push(Row::new(vec![key, summary, status]));
    }
    (rows, keys)
}
