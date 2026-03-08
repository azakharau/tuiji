use std::sync::Arc;

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

pub struct IssuesTableState {
    rows: Vec<TableRow<'static>>,
    selected_index: usize,
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
        Self {
            rows: build_rows(issues),
            selected_index: 0,
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

pub struct IssuesTableController;

impl IssuesTableController {
    pub fn handle_command(state: &mut IssuesTableState, command: Command) -> ScreenState {
        match command.action {
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

pub struct IssuesTableView;

impl IssuesTableView {
    pub fn draw(
        frame: &mut Frame,
        state: &IssuesTableState,
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

        if state.is_empty() {
            frame.render_widget(
                EmptyState::new(state.empty_title(), state.empty_message(), context),
                layout[1],
            );
        } else {
            let table = TableView {
                rows: state.rows(),
                selected: state.selected_index(),
                context,
            };
            table.render(frame, layout[1]);
        }

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);
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
        rows.push(ratatui::widgets::Row::new(vec![key, summary, status]));
    }
    rows
}
