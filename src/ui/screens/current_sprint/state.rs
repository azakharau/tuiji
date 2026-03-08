use crate::data::IssueSummary;

pub struct CurrentSprintState {
    issues: Vec<IssueSummary>,
    selected_index: usize,
    scroll_offset: usize,
    rows_visible: usize,
    detail_open: bool,
}

impl CurrentSprintState {
    pub fn new(issues: Vec<IssueSummary>) -> Self {
        Self {
            issues,
            selected_index: 0,
            scroll_offset: 0,
            rows_visible: 1,
            detail_open: false,
        }
    }

    pub fn issues(&self) -> &[IssueSummary] {
        &self.issues
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn rows_visible(&self) -> usize {
        self.rows_visible
    }

    pub fn detail_open(&self) -> bool {
        self.detail_open
    }

    pub fn selected_issue(&self) -> Option<&IssueSummary> {
        self.issues.get(self.selected_index)
    }

    pub fn set_rows_visible(&mut self, rows_visible: usize) {
        self.rows_visible = rows_visible.max(1);
    }

    pub fn set_selected_index(&mut self, value: usize) {
        self.selected_index = value;
    }

    pub fn set_scroll_offset(&mut self, value: usize) {
        self.scroll_offset = value;
    }

    pub fn toggle_detail(&mut self) {
        if !self.issues.is_empty() {
            self.detail_open = !self.detail_open;
        }
    }

    pub fn close_detail(&mut self) {
        self.detail_open = false;
    }
}
