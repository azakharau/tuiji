use crate::{data::IssueSummary, ui::screens::issues_table::IssuesTableState};

pub struct MyIssuesState {
    table: IssuesTableState,
    error: Option<String>,
}

impl MyIssuesState {
    pub fn new(issues: Vec<IssueSummary>, error: Option<String>) -> Self {
        Self {
            table: IssuesTableState::my_issues(issues),
            error,
        }
    }

    pub fn table(&self) -> &IssuesTableState {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut IssuesTableState {
        &mut self.table
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
