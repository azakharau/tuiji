use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Cell, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use crate::ui::context::RenderContext;

use super::{formatting::format_story_points, state::CurrentSprintState};

pub struct TableView;

impl TableView {
    pub fn draw(
        frame: &mut Frame,
        area: Rect,
        state: &CurrentSprintState,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        let highlight_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().selection)
            .add_modifier(Modifier::BOLD);

        let block = Block::bordered()
            .title("Current Sprint")
            .style(base_style)
            .border_style(Style::default().fg(context.colors().border))
            .padding(Padding::horizontal(1));

        let table_area = if area.width > 1 {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height,
            }
        } else {
            area
        };

        let start = state.scroll_offset();
        let end = (start + state.rows_visible()).min(state.issues().len());
        let rows = state.issues()[start..end]
            .iter()
            .map(|issue| {
                let story_points = issue
                    .story_points
                    .map(format_story_points)
                    .unwrap_or_else(|| "-".to_string());
                let meta = format!(
                    "{} comments{}{}",
                    issue.comments.len(),
                    if issue.dirty { "  dirty" } else { "" },
                    if issue.conflict { "  conflict" } else { "" }
                );
                Row::new(vec![
                    Cell::from(issue.key.clone()),
                    Cell::from(issue.issue_type.clone()),
                    Cell::from(Text::from(vec![
                        Line::from(issue.summary.clone()),
                        Line::from(meta),
                    ])),
                    Cell::from(issue.status.clone()),
                    Cell::from(story_points),
                    Cell::from(issue.priority.clone()),
                    Cell::from(issue.assignee.clone()),
                ])
                .height(2)
            })
            .collect::<Vec<_>>();

        let mut table_state = TableState::default();
        if !rows.is_empty() {
            table_state.select(Some(state.selected_index().saturating_sub(start)));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Length(11),
                Constraint::Length(4),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new([
                Span::styled("Key", Style::default().fg(context.colors().accent)),
                Span::styled("Type", Style::default().fg(context.colors().accent)),
                Span::styled("Summary", Style::default().fg(context.colors().accent)),
                Span::styled("Status", Style::default().fg(context.colors().accent)),
                Span::styled("SP", Style::default().fg(context.colors().accent)),
                Span::styled("Priority", Style::default().fg(context.colors().accent)),
                Span::styled("Assignee", Style::default().fg(context.colors().accent)),
            ])
            .bottom_margin(1),
        )
        .block(block)
        .style(base_style)
        .row_highlight_style(highlight_style);

        frame.render_stateful_widget(table, table_area, &mut table_state);

        if state.issues().len() > state.rows_visible() {
            let mut scrollbar_state =
                ScrollbarState::new(state.issues().len()).position(state.selected_index());
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(context.colors().accent));
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }
}
