use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{data::DiffEntry, ui::context::RenderContext};

use super::super::state::{CommentDiff, ConflictsState};

pub(super) fn render_issue_diff(
    frame: &mut Frame,
    state: &ConflictsState,
    area: Rect,
    context: &RenderContext,
) {
    let rows = build_diff_rows(state.issue_diffs(), context, None);
    render_diff_table(frame, "Issue diff", rows, area, context);
}

pub(super) fn render_comment_diffs(
    frame: &mut Frame,
    diffs: &[CommentDiff],
    area: Rect,
    context: &RenderContext,
) {
    let rows = build_comment_rows(diffs, context);
    render_diff_table(frame, "Comment diff", rows, area, context);
}

fn render_diff_table(
    frame: &mut Frame,
    title: &str,
    rows: Vec<Row<'static>>,
    area: Rect,
    context: &RenderContext,
) {
    let header =
        Row::new(["Field", "Local", "Remote"]).style(Style::default().fg(context.colors().accent));
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(context.colors().border))
        .title(Line::from(title).centered())
        .title_style(Style::default().fg(context.colors().accent));

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Percentage(45),
            Constraint::Percentage(41),
        ],
    )
    .header(header)
    .block(block)
    .style(
        Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background),
    );
    frame.render_widget(table, area);
}

fn build_diff_rows(
    diffs: &[DiffEntry],
    context: &RenderContext,
    label: Option<String>,
) -> Vec<Row<'static>> {
    let mut rows = Vec::new();
    if let Some(label) = label {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                label,
                Style::default().fg(context.colors().accent),
            )),
            Cell::from(""),
            Cell::from(""),
        ]));
    }
    if diffs.is_empty() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "No changes",
                Style::default().fg(context.colors().border),
            )),
            Cell::from(""),
            Cell::from(""),
        ]));
        return rows;
    }

    for diff in diffs {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                diff.field,
                Style::default().fg(context.colors().accent),
            )),
            Cell::from(Span::styled(
                diff.local.clone(),
                Style::default().fg(context.colors().text),
            )),
            Cell::from(Span::styled(
                diff.remote.clone(),
                Style::default().fg(context.colors().warning),
            )),
        ]));
    }
    rows
}

fn build_comment_rows(diffs: &[CommentDiff], context: &RenderContext) -> Vec<Row<'static>> {
    if diffs.is_empty() {
        return build_diff_rows(&[], context, None);
    }
    let mut rows = Vec::new();
    for diff in diffs {
        let label = format!("Comment {}", diff.id);
        rows.extend(build_diff_rows(&diff.diffs, context, Some(label)));
    }
    rows
}
