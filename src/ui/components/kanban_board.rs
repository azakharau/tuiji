use std::{collections::HashMap, rc::Rc, sync::Arc};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{
    client::jira::BoardConfig,
    ui::{
        components::{badges::{StatusBadge, StatusVariant}, issue_card::IssueCardComponent},
        context::RenderContext,
    },
};

pub struct KanbanBoard<'a> {
    pub id: u32,
    pub title: String,
    pub issues: Arc<Vec<IssueCardComponent>>,
    pub board_cfg: &'a BoardConfig,
    pub context: &'a RenderContext,
    pub selected_col: usize,
    pub selected_row: usize,
    pub scroll_offset: usize,
    pub rows_visible: usize,
}

impl<'a> KanbanBoard<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        title: String,
        issues: Arc<Vec<IssueCardComponent>>,
        cfg: &'a BoardConfig,
        context: &'a RenderContext,
        selected_col: usize,
        selected_row: usize,
        scroll_offset: usize,
        rows_visible: usize,
    ) -> Self {
        KanbanBoard {
            id,
            title,
            issues,
            board_cfg: cfg,
            context,
            selected_col,
            selected_row,
            scroll_offset,
            rows_visible,
        }
    }

    fn cols_layout(&self) -> Layout {
        let constraints: Vec<Constraint> = (0..self.board_cfg.columns.len())
            .map(|_| Constraint::Ratio(1, self.board_cfg.columns.len() as u32))
            .collect();
        Layout::horizontal(constraints)
    }

    fn rows_layout(&self) -> Layout {
        let mut constraints: Vec<Constraint> = Vec::new();
        constraints.push(Constraint::Length(1)); // header
        let issue_height = self.issues.first().map(|i| i.height()).unwrap_or(8);
        (0..self.rows_visible).for_each(|_| {
            constraints.push(Constraint::Length(issue_height));
        });
        Layout::vertical(constraints)
    }

    fn table(&self, area: Rect) -> Vec<Rc<[Rect]>> {
        let cols_layout = self.cols_layout().split(area);
        let mut layout: Vec<Rc<[Rect]>> = Vec::with_capacity(cols_layout.len());

        cols_layout.iter().for_each(|c| {
            let rows_layout = self.rows_layout().split(*c);
            layout.push(rows_layout);
        });

        layout
    }

    fn group_issues_by_status(&self) -> HashMap<String, Vec<IssueCardComponent>> {
        let mut grouped_issues: HashMap<String, Vec<IssueCardComponent>> = HashMap::new();

        for issue in self.issues.clone().iter() {
            grouped_issues
                .entry(issue.status.clone())
                .or_default()
                .push(issue.clone());
        }

        grouped_issues
    }
}

impl Widget for KanbanBoard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let base_style = Style::default()
            .fg(self.context.colors().text)
            .bg(self.context.colors().background);
        let table = self.table(area);
        let header_index = 0;
        let _footer_index = table.len() - 1;
        let grouped_issues = self.group_issues_by_status();
        let empty: &[IssueCardComponent] = &[];
        for (i, col) in self.board_cfg.columns.iter().enumerate() {
            let badge = StatusBadge::new(
                col.name.as_str(),
                StatusVariant::Custom(col.name.as_str()),
                self.context,
            );
            badge.render(table[i][header_index], buf);
            let column_issues = grouped_issues
                .get(&col.name)
                .map(|v| v.as_slice())
                .unwrap_or(empty);
            let offset = self.scroll_offset;
            for row_idx in 0..self.rows_visible {
                let area = table[i][row_idx + 1];
                if let Some(issue) = column_issues.get(offset + row_idx) {
                    let selected =
                        self.selected_col == i && (offset + row_idx) == self.selected_row;
                    issue.render_with_selection(area, buf, selected, self.context);
                } else {
                    // Clear the area for empty rows to avoid ghost content.
                    Paragraph::new(Text::raw("")).style(base_style).render(area, buf);
                }
            }
        }
    }
}
