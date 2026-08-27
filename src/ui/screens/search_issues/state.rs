use tui_input::{Input, InputRequest};

use crate::{data::IssueSummary, ui::screens::issues_table::IssuesTableState};

pub struct SearchIssuesState {
    table: IssuesTableState,
    input: Input,
    active_query: String,
    load_error: Option<String>,
    input_error: Option<String>,
}

impl SearchIssuesState {
    pub fn new(query: String, issues: Vec<IssueSummary>, load_error: Option<String>) -> Self {
        Self {
            table: IssuesTableState::search_issues(issues),
            input: Input::new(query.clone()),
            active_query: query,
            load_error,
            input_error: None,
        }
    }

    pub fn table(&self) -> &IssuesTableState {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut IssuesTableState {
        &mut self.table
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn active_query(&self) -> &str {
        &self.active_query
    }

    pub fn error(&self) -> Option<&str> {
        self.input_error.as_deref().or(self.load_error.as_deref())
    }

    pub fn edit(&mut self, request: InputRequest) {
        self.input.handle(request);
        self.input_error = None;
    }

    pub fn submit_query(&mut self) -> Option<String> {
        let query = self.input.value().trim();
        if query.is_empty() {
            self.input_error = Some("Enter a JQL query.".to_string());
            None
        } else {
            self.input_error = None;
            Some(query.to_string())
        }
    }
}
