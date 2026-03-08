use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{contracts::sync::SyncJob, ui::context::RenderContext};

use super::{
    super::state::SyncStatusState,
    formatting::{format_job_kind, format_job_source, format_next_attempt, format_time},
};

pub(super) fn render_queue_table(
    frame: &mut Frame,
    area: Rect,
    state: &SyncStatusState,
    context: &RenderContext,
) {
    let snapshot = state.snapshot();
    let mut rows = Vec::new();
    if let Some(active) = snapshot.active.as_ref() {
        rows.push(queue_row("active", active, context));
    }
    if snapshot.queue_entries.is_empty() && snapshot.active.is_none() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "No queued jobs",
                Style::default().fg(context.colors().border),
            )),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]));
    } else {
        for job in &snapshot.queue_entries {
            rows.push(queue_row("queued", job, context));
        }
    }

    let header = Row::new([
        "State",
        "Kind",
        "Source",
        "Created At",
        "Retries",
        "Next Attempt",
    ])
    .style(Style::default().fg(context.colors().accent));
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(20),
            Constraint::Length(7),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from("Queue").centered())
            .title_style(Style::default().fg(context.colors().accent))
            .style(Style::default().fg(context.colors().border)),
    )
    .style(
        Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background),
    );
    frame.render_widget(table, area);
}

fn queue_row(state: &str, job: &SyncJob, context: &RenderContext) -> Row<'static> {
    Row::new(vec![
        Cell::from(Span::styled(
            state.to_string(),
            Style::default().fg(context.colors().accent),
        )),
        Cell::from(format_job_kind(job.kind)),
        Cell::from(format_job_source(job.source)),
        Cell::from(format_time(Some(job.created_at))),
        Cell::from(job.retries.to_string()),
        Cell::from(format_next_attempt(job.next_attempt_at)),
    ])
}
