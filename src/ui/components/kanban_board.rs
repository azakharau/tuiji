use std::{collections::HashMap, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{client::jira::BoardConfig, ui::components::issue_card::IssueCardComponent};

pub struct KanbanBoard<'a> {
    pub id: u32,
    pub title: String,
    pub issues: Rc<Vec<IssueCardComponent>>,
    pub board_cfg: &'a BoardConfig,
}

impl<'a> KanbanBoard<'a> {
    pub fn new(
        id: u32,
        title: String,
        issues: Rc<Vec<IssueCardComponent>>,
        cfg: &'a BoardConfig,
    ) -> Self {
        KanbanBoard {
            id,
            title,
            issues,
            board_cfg: cfg,
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
        let rows_count = self.max_issues_in_status();
        let sample = if let Some(issue) = self.issues.first() {
            issue.to_owned()
        } else {
            IssueCardComponent::new(
                "".to_string(),
                "No Issues".to_string(),
                None,
                "".to_string(),
                crate::ui::components::issue_card::IssueType::Task,
                "".to_string(),
                crate::ui::components::issue_card::Priority::Medium,
                None,
            )
        };
        constraints.push(Constraint::Length(1));

        (0..rows_count).for_each(|_| {
            constraints.push(Constraint::Length(sample.height()));
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

    fn max_issues_in_status(&self) -> usize {
        let grouped_issues = self.group_issues_by_status();
        grouped_issues
            .values()
            .map(|issues| issues.len())
            .max()
            .unwrap_or(0)
    }
}

impl Widget for KanbanBoard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let table = self.table(area);
        let header_index = 0;
        let _footer_index = table.len() - 1;
        let grouped_issues = self.group_issues_by_status();
        for (i, col) in self.board_cfg.columns.iter().enumerate() {
            Paragraph::new(Text::from(col.name.to_uppercase()))
                .alignment(Alignment::Center)
                .render(table[i][header_index], buf);
            let body_len = grouped_issues.get(&col.name).unwrap_or(&Vec::new()).len();
            let body_indexes = 0..body_len;
            for body_index in body_indexes {
                grouped_issues.get(&col.name).unwrap_or(&Vec::new())[body_index]
                    .to_owned()
                    .render(table[i][body_index + 1], buf);
            }
        }
    }
}
