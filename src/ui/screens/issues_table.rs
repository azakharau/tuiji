use std::{ops::Range, sync::Arc};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use crate::{
    data::IssueSummary,
    ui::{
        components::{
            bottom_bar::BottomBar,
            list::{EmptyState, TableRow, TableView},
        },
        context::RenderContext,
        screens::ScreenState,
    },
    ui::{
        interaction::Mode,
        interaction::{ActionHint, ActionId, Command},
    },
};

pub struct IssueWorkspaceState {
    issues: Vec<IssueSummary>,
    selected_index: usize,
    scroll_offset: usize,
    rows_visible: usize,
}

impl IssueWorkspaceState {
    pub fn new(issues: Vec<IssueSummary>) -> Self {
        let mut workspace = Self {
            issues,
            selected_index: 0,
            scroll_offset: 0,
            rows_visible: 1,
        };
        workspace.clamp_viewport();
        workspace
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

    pub fn selected_issue(&self) -> Option<&IssueSummary> {
        self.issues.get(self.selected_index)
    }

    pub fn visible_range(&self) -> Range<usize> {
        let start = self.scroll_offset.min(self.issues.len());
        let end = (start + self.rows_visible).min(self.issues.len());
        start..end
    }

    pub fn set_rows_visible(&mut self, rows_visible: usize) {
        self.rows_visible = rows_visible.max(1);
        self.clamp_viewport();
    }

    pub fn move_up(&mut self, n: usize) {
        if self.is_empty() {
            return;
        }
        self.selected_index = self.selected_index.saturating_sub(n.max(1));
        self.clamp_viewport();
    }

    pub fn move_down(&mut self, n: usize) {
        if self.is_empty() {
            return;
        }
        let max_index = self.issues.len() - 1;
        self.selected_index = (self.selected_index + n.max(1)).min(max_index);
        self.clamp_viewport();
    }

    pub fn move_top(&mut self) {
        if self.is_empty() {
            return;
        }
        self.selected_index = 0;
        self.clamp_viewport();
    }

    pub fn move_bottom(&mut self) {
        if self.is_empty() {
            return;
        }
        self.selected_index = self.issues.len() - 1;
        self.clamp_viewport();
    }

    fn clamp_viewport(&mut self) {
        if self.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }

        self.selected_index = self.selected_index.min(self.issues.len() - 1);
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.rows_visible {
            self.scroll_offset = self.selected_index + 1 - self.rows_visible;
        }

        let max_offset = self.issues.len().saturating_sub(self.rows_visible);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }
}

pub struct IssuesTableState {
    workspace: IssueWorkspaceState,
    rows: Vec<TableRow<'static>>,
    title: &'static str,
    empty_title: &'static str,
    empty_message: &'static str,
}

impl IssuesTableState {
    pub fn my_issues(issues: Vec<IssueSummary>) -> Self {
        Self::new("My Issues", "No issues", "Sync to load issues.", issues)
    }

    pub fn search_issues(issues: Vec<IssueSummary>) -> Self {
        Self::new("Search Issues", "No issues", "Sync to load issues.", issues)
    }

    pub fn new(
        title: &'static str,
        empty_title: &'static str,
        empty_message: &'static str,
        issues: Vec<IssueSummary>,
    ) -> Self {
        let rows = build_rows(&issues);
        Self {
            workspace: IssueWorkspaceState::new(issues),
            rows,
            title,
            empty_title,
            empty_message,
        }
    }

    pub fn title(&self) -> &'static str {
        self.title
    }

    pub fn empty_title(&self) -> &'static str {
        self.empty_title
    }

    pub fn empty_message(&self) -> &'static str {
        self.empty_message
    }

    pub fn rows(&self) -> &[TableRow<'static>] {
        &self.rows
    }

    pub fn visible_rows(&self) -> &[TableRow<'static>] {
        let range = self.workspace.visible_range();
        &self.rows[range]
    }

    pub fn issues(&self) -> &[IssueSummary] {
        self.workspace.issues()
    }

    pub fn is_empty(&self) -> bool {
        self.workspace.is_empty()
    }

    pub fn selected_index(&self) -> usize {
        self.workspace.selected_index()
    }

    pub fn visible_selected_index(&self) -> usize {
        self.selected_index()
            .saturating_sub(self.workspace.scroll_offset())
    }

    pub fn scroll_offset(&self) -> usize {
        self.workspace.scroll_offset()
    }

    pub fn rows_visible(&self) -> usize {
        self.workspace.rows_visible()
    }

    pub fn selected_issue(&self) -> Option<&IssueSummary> {
        self.workspace.selected_issue()
    }

    pub fn set_rows_visible(&mut self, rows_visible: usize) {
        self.workspace.set_rows_visible(rows_visible);
    }

    pub fn move_up(&mut self, n: usize) {
        self.workspace.move_up(n);
    }

    pub fn move_down(&mut self, n: usize) {
        self.workspace.move_down(n);
    }

    pub fn move_top(&mut self) {
        self.workspace.move_top();
    }

    pub fn move_bottom(&mut self) {
        self.workspace.move_bottom();
    }
}

pub struct IssueWorkspaceController;

impl IssueWorkspaceController {
    pub fn handle_command(state: &mut IssueWorkspaceState, command: Command) -> ScreenState {
        match command.action {
            ActionId::Confirm => state
                .selected_issue()
                .map(|issue| ScreenState::ViewIssue(issue.key.clone()))
                .unwrap_or(ScreenState::Stay),
            ActionId::Refresh => ScreenState::Refresh,
            ActionId::MoveUp => {
                state.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                state.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.move_bottom();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}

pub struct IssuesTableController;

impl IssuesTableController {
    pub fn handle_command(state: &mut IssuesTableState, command: Command) -> ScreenState {
        IssueWorkspaceController::handle_command(&mut state.workspace, command)
    }
}

pub struct IssuesTableView;

impl IssuesTableView {
    pub fn draw(
        frame: &mut Frame,
        state: &mut IssuesTableState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(
            ratatui::widgets::Block::default().style(base_style),
            frame.area(),
        );
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let title = Paragraph::new(Line::from(state.title()).centered())
            .style(Style::default().fg(context.colors().accent))
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        let rows_visible = layout[1].height.saturating_sub(1) as usize;
        state.set_rows_visible(rows_visible);

        if state.is_empty() {
            frame.render_widget(
                EmptyState::new(state.empty_title(), state.empty_message(), context),
                layout[1],
            );
        } else {
            let table = TableView {
                rows: state.visible_rows(),
                selected: state.visible_selected_index(),
                context,
            };
            table.render(frame, layout[1]);
        }

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);
    }
}

fn build_rows(issues: &[IssueSummary]) -> Vec<TableRow<'static>> {
    issues
        .iter()
        .map(|issue| {
            ratatui::widgets::Row::new(vec![
                issue.key.clone(),
                issue.summary.clone(),
                issue.status.clone(),
            ])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(key: &str) -> IssueSummary {
        IssueSummary {
            key: key.to_string(),
            summary: format!("Summary for {key}"),
            epic: None,
            status: "TODO".to_string(),
            issue_type: "Task".to_string(),
            assignee: "Alex".to_string(),
            priority: "Medium".to_string(),
            story_points: None,
            project_key: Some("TEST".to_string()),
            sprint_id: Some(1),
            updated_at: None,
            comments: Vec::new(),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: None,
            reporter: None,
            creator: None,
            created_at: None,
            resolution_date: None,
            resolution: None,
            labels: Vec::new(),
            fix_versions: Vec::new(),
            parent_key: None,
            environment: None,
            time_estimate: None,
            time_spent: None,
            time_remaining: None,
            custom_fields: None,
        }
    }

    #[test]
    fn workspace_should_keep_selection_visible_when_moving_to_bottom() {
        let mut workspace = IssueWorkspaceState::new(vec![
            issue("TEST-1"),
            issue("TEST-2"),
            issue("TEST-3"),
            issue("TEST-4"),
        ]);
        workspace.set_rows_visible(2);

        workspace.move_bottom();

        assert_eq!(workspace.selected_index(), 3);
        assert_eq!(workspace.scroll_offset(), 2);
        assert_eq!(workspace.visible_range(), 2..4);
    }

    #[test]
    fn workspace_should_clamp_scroll_when_rows_visible_grow() {
        let mut workspace = IssueWorkspaceState::new(vec![
            issue("TEST-1"),
            issue("TEST-2"),
            issue("TEST-3"),
            issue("TEST-4"),
            issue("TEST-5"),
        ]);
        workspace.set_rows_visible(2);
        workspace.move_bottom();

        workspace.set_rows_visible(4);

        assert_eq!(workspace.selected_index(), 4);
        assert_eq!(workspace.scroll_offset(), 1);
        assert_eq!(workspace.visible_range(), 1..5);
    }

    #[test]
    fn issues_table_confirm_should_open_selected_issue() {
        let mut state = IssuesTableState::my_issues(vec![issue("TEST-1")]);

        let result = IssuesTableController::handle_command(
            &mut state,
            Command {
                action: ActionId::Confirm,
                repeat: 1,
            },
        );

        assert_eq!(result, ScreenState::ViewIssue("TEST-1".to_string()));
        assert_eq!(
            state.selected_issue().map(|issue| issue.key.as_str()),
            Some("TEST-1")
        );
    }

    #[test]
    fn issues_table_selection_should_follow_workspace_navigation() {
        let mut state = IssuesTableState::search_issues(vec![
            issue("TEST-1"),
            issue("TEST-2"),
            issue("TEST-3"),
        ]);
        state.set_rows_visible(2);

        let result = IssuesTableController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveBottom,
                repeat: 1,
            },
        );

        assert_eq!(result, ScreenState::Refresh);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.scroll_offset(), 1);
        assert_eq!(state.visible_selected_index(), 1);
    }
}
