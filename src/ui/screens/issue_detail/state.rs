use crate::data::IssueSummary;
use ratatui::text::Line;

pub struct IssueDetailState {
    issue: IssueSummary,
    scroll_offset: usize,
    max_scroll: usize,
    viewport_height: usize,
    horizontal_offset: usize,
    cached_content: Option<Vec<Line<'static>>>,
}

impl IssueDetailState {
    pub fn new(issue: IssueSummary) -> Self {
        Self {
            issue,
            scroll_offset: 0,
            max_scroll: 0,
            viewport_height: 1,
            horizontal_offset: 0,
            cached_content: None,
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

    /// Get cached content lines, or None if not cached
    pub fn cached_content(&self) -> Option<&Vec<Line<'static>>> {
        self.cached_content.as_ref()
    }

    /// Cache content lines for rendering optimization
    pub fn cache_content(&mut self, lines: Vec<Line<'static>>) {
        self.cached_content = Some(lines);
    }

    /// Invalidate cache (e.g., when issue is updated)
    pub fn invalidate_cache(&mut self) {
        self.cached_content = None;
    }

    /// Get horizontal scroll offset
    pub fn horizontal_offset(&self) -> usize {
        self.horizontal_offset
    }

    /// Scroll right (increase horizontal offset)
    pub fn scroll_right(&mut self) {
        self.horizontal_offset += 4; // Scroll by 4 characters
    }

    /// Scroll left (decrease horizontal offset)
    pub fn scroll_left(&mut self) {
        self.horizontal_offset = self.horizontal_offset.saturating_sub(4);
    }

    /// Reset horizontal scroll to start
    pub fn reset_horizontal_scroll(&mut self) {
        self.horizontal_offset = 0;
    }
}
