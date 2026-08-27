use ratatui::text::Line;
use tui_input::{Input, InputRequest};

use crate::data::{IssueSummary, TransitionChoice};

pub struct IssueDetailState {
    issue: Option<IssueSummary>,
    unavailable_message: Option<String>,
    browse_url: Option<String>,
    scroll_offset: usize,
    max_scroll: usize,
    viewport_height: usize,
    horizontal_offset: usize,
    cached_content: Option<Vec<Line<'static>>>,
    comment_input: Option<Input>,
    transitions: Vec<TransitionChoice>,
    transitions_loaded: bool,
    transitions_requested: bool,
    transition_error: Option<String>,
    transition_picker_open: bool,
    transition_selected: usize,
}

impl IssueDetailState {
    pub fn new(
        issue: IssueSummary,
        base_url: Option<String>,
        transition_result: Option<Result<Vec<TransitionChoice>, String>>,
    ) -> Self {
        let browse_url = base_url
            .map(|base_url| format!("{}/browse/{}", base_url.trim_end_matches('/'), issue.key));
        let (transitions, transitions_loaded, transition_error, transition_picker_open) =
            match transition_result {
                Option::Some(Ok(transitions)) => (transitions, true, Option::None, true),
                Option::Some(Err(error)) => (Vec::new(), true, Some(error), true),
                Option::None => (Vec::new(), false, Option::None, false),
            };

        Self {
            issue: Some(issue),
            unavailable_message: None,
            browse_url,
            scroll_offset: 0,
            max_scroll: 0,
            viewport_height: 1,
            horizontal_offset: 0,
            cached_content: None,
            comment_input: None,
            transitions,
            transitions_loaded,
            transitions_requested: false,
            transition_error,
            transition_picker_open,
            transition_selected: 0,
        }
    }

    pub fn unavailable(message: String) -> Self {
        Self {
            issue: None,
            unavailable_message: Some(message),
            browse_url: None,
            scroll_offset: 0,
            max_scroll: 0,
            viewport_height: 1,
            horizontal_offset: 0,
            cached_content: None,
            comment_input: None,
            transitions: Vec::new(),
            transitions_loaded: false,
            transitions_requested: false,
            transition_error: None,
            transition_picker_open: false,
            transition_selected: 0,
        }
    }

    pub fn issue(&self) -> Option<&IssueSummary> {
        self.issue.as_ref()
    }

    pub fn issue_key(&self) -> Option<&str> {
        self.issue.as_ref().map(|issue| issue.key.as_str())
    }

    pub fn unavailable_message(&self) -> Option<&str> {
        self.unavailable_message.as_deref()
    }

    pub fn browse_url(&self) -> Option<&str> {
        self.browse_url.as_deref()
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
        let page_size = self.viewport_height.max(1);
        self.scroll_offset = (self.scroll_offset + page_size).min(self.max_scroll);
    }

    pub fn page_up(&mut self) {
        let page_size = self.viewport_height.max(1);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn update_bounds(&mut self, total_lines: usize, viewport_height: usize) {
        self.viewport_height = viewport_height.max(1);
        self.max_scroll = total_lines.saturating_sub(viewport_height);
        self.scroll_offset = self.scroll_offset.min(self.max_scroll);
    }

    pub fn scroll_percentage(&self) -> usize {
        (self.scroll_offset * 100)
            .checked_div(self.max_scroll)
            .unwrap_or(100)
    }

    pub fn cached_content(&self) -> Option<&Vec<Line<'static>>> {
        self.cached_content.as_ref()
    }

    pub fn cache_content(&mut self, lines: Vec<Line<'static>>) {
        self.cached_content = Some(lines);
    }

    pub fn horizontal_offset(&self) -> usize {
        self.horizontal_offset
    }

    pub fn scroll_right(&mut self) {
        self.horizontal_offset = self.horizontal_offset.saturating_add(4);
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_offset = self.horizontal_offset.saturating_sub(4);
    }

    pub fn reset_horizontal_scroll(&mut self) {
        self.horizontal_offset = 0;
    }

    pub fn open_comment_input(&mut self) {
        self.comment_input = Some(Input::default());
    }

    pub fn close_comment_input(&mut self) {
        self.comment_input = None;
    }

    pub fn comment_input(&self) -> Option<&Input> {
        self.comment_input.as_ref()
    }

    pub fn handle_comment_input(&mut self, request: InputRequest) {
        if let Some(input) = self.comment_input.as_mut() {
            input.handle(request);
        }
    }

    pub fn take_comment(&mut self) -> Option<String> {
        let body = self.comment_input.as_ref()?.value().trim().to_string();
        if body.is_empty() {
            return None;
        }
        self.comment_input = None;
        Some(body)
    }

    pub fn request_transitions(&mut self) {
        if self.transitions_loaded {
            self.transition_picker_open = true;
        } else {
            self.transitions_requested = true;
        }
    }

    pub fn transitions_requested(&self) -> bool {
        self.transitions_requested
    }

    pub fn transition_picker_open(&self) -> bool {
        self.transition_picker_open
    }

    pub fn transition_error(&self) -> Option<&str> {
        self.transition_error.as_deref()
    }

    pub fn transitions(&self) -> &[TransitionChoice] {
        &self.transitions
    }

    pub fn transition_selected(&self) -> usize {
        self.transition_selected
    }

    pub fn select_previous_transition(&mut self) {
        self.transition_selected = self.transition_selected.saturating_sub(1);
    }

    pub fn select_next_transition(&mut self) {
        if !self.transitions.is_empty() {
            self.transition_selected =
                (self.transition_selected + 1).min(self.transitions.len() - 1);
        }
    }

    pub fn selected_transition(&self) -> Option<&TransitionChoice> {
        self.transitions.get(self.transition_selected)
    }

    pub fn close_transition_picker(&mut self) {
        self.transition_picker_open = false;
    }
}
