use crate::data::IssueSummary;

pub struct IssueDetailState {
    issue: IssueSummary,
    scroll_offset: usize,
    max_scroll: usize,
    viewport_height: usize,
}

impl IssueDetailState {
    pub fn new(issue: IssueSummary) -> Self {
        Self {
            issue,
            scroll_offset: 0,
            max_scroll: 0,
            viewport_height: 1,
        }
    }

    pub fn issue(&self) -> &IssueSummary {
        &self.issue
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = (self.scroll_offset + 1).min(self.max_scroll);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll;
    }

    pub fn page_down(&mut self) {
        let page_size = self.viewport_height.saturating_sub(2).max(1);
        self.scroll_offset = (self.scroll_offset + page_size).min(self.max_scroll);
    }

    pub fn page_up(&mut self) {
        let page_size = self.viewport_height.saturating_sub(2).max(1);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn update_bounds(&mut self, total_lines: usize, viewport_height: usize) {
        self.viewport_height = viewport_height.max(1);
        // Max scroll is total lines minus viewport height, but at least 0
        self.max_scroll = total_lines.saturating_sub(viewport_height);
        // Clamp current offset to valid range
        self.scroll_offset = self.scroll_offset.min(self.max_scroll);
    }

    /// Get scroll position as a percentage (0-100)
    pub fn scroll_percentage(&self) -> usize {
        if self.max_scroll == 0 {
            100 // All content visible, consider it 100%
        } else {
            (self.scroll_offset * 100) / self.max_scroll
        }
    }
}
