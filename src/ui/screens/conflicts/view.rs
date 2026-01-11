use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, ListItem, Paragraph, Row, Table},
};

use crate::{
    app::{key_handlers::ActionHint, state::Mode},
    ui::{
        components::{
            bottom_bar::BottomBar,
            layout::ModalFrame,
            list::{EmptyState, ListView},
        },
        context::RenderContext,
    },
};

use super::state::{CommentDiff, ConflictsState, PendingResolve};

pub struct ConflictsView;

impl ConflictsView {
    pub fn draw(
        frame: &mut Frame,
        state: &ConflictsState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());

        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        render_header(frame, layout[0], context);

        if state.is_empty() {
            frame.render_widget(
                EmptyState::new("No conflicts", "You're up to date.", context),
                layout[1],
            );
        } else {
            let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(layout[1]);

            let list_items = build_list_items(state, context);
            let list = ListView {
                items: list_items.as_slice(),
                selected: state.selected_index(),
                context,
            };
            list.render(frame, body[0]);

            let right = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(body[1]);

            render_issue_diff(frame, state, right[0], context);
            render_comment_diffs(frame, state.comment_diffs(), right[1], context);
        }

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);

        if let Some(pending) = state.pending_resolve() {
            render_confirm_modal(frame, pending, context);
        }
    }
}

fn render_header(frame: &mut Frame, area: Rect, context: &RenderContext) {
    let header = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

    let title = Paragraph::new(Line::from("Conflicts").centered())
        .style(Style::default().fg(context.colors().accent))
        .alignment(Alignment::Center);
    frame.render_widget(title, header[0]);

    let hint_line = Line::from(vec![
        Span::styled("L", Style::default().fg(context.colors().accent)),
        Span::styled(
            " = Use Local  ",
            Style::default().fg(context.colors().border),
        ),
        Span::styled("J", Style::default().fg(context.colors().accent)),
        Span::styled(" = Use Jira", Style::default().fg(context.colors().border)),
    ])
    .centered();
    frame.render_widget(
        Paragraph::new(hint_line).alignment(Alignment::Center),
        header[1],
    );
}

fn render_issue_diff(
    frame: &mut Frame,
    state: &ConflictsState,
    area: Rect,
    context: &RenderContext,
) {
    let rows = build_diff_rows(state.issue_diffs(), context, None);
    render_diff_table(frame, "Issue diff", rows, area, context);
}

fn render_comment_diffs(
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
    diffs: &[crate::data::DiffEntry],
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

fn build_list_items(state: &ConflictsState, context: &RenderContext) -> Vec<ListItem<'static>> {
    let mut items = Vec::with_capacity(state.issues().len());
    for issue in state.issues() {
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{} {}", issue.key, issue.summary),
            Style::default().fg(context.colors().text),
        ));
        if issue.conflict {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                "[Issue]",
                Style::default().fg(context.colors().accent),
            ));
        }
        let comment_count = issue.comments.iter().filter(|c| c.conflict).count();
        if comment_count > 0 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[Comments ({comment_count})]"),
                Style::default().fg(context.colors().warning),
            ));
        }
        items.push(ListItem::new(Line::from(spans)));
    }
    items
}

fn render_confirm_modal(frame: &mut Frame, pending: PendingResolve, context: &RenderContext) {
    let area = frame.area();
    let width = (area.width.saturating_mul(6) / 10).max(28).min(area.width);
    let height = 7.min(area.height);
    let modal = crate::app::input::overlay::modal_area(area, width, height);
    let inner = ModalFrame::new(
        "Confirm",
        modal,
        Style::default().fg(context.colors().accent),
        context,
    )
    .render(frame);

    let action = match pending {
        PendingResolve::Local => "Use Local",
        PendingResolve::Remote => "Use Jira",
    };

    let text = vec![
        Line::from(Span::styled(
            format!("{action}?"),
            Style::default().fg(context.colors().text),
        ))
        .centered(),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(context.colors().accent)),
            Span::styled(" = Yes    ", Style::default().fg(context.colors().border)),
            Span::styled("q", Style::default().fg(context.colors().accent)),
            Span::styled(" = No", Style::default().fg(context.colors().border)),
        ])
        .centered(),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}
