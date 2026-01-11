use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    app::state::Mode,
    data::SyncLogFilter,
    ui::{components::bottom_bar::BottomBar, context::RenderContext},
};

use super::state::SyncStatusState;

pub struct SyncStatusView;

impl SyncStatusView {
    pub fn draw(
        frame: &mut Frame,
        state: &SyncStatusState,
        mode: Mode,
        actions: &std::sync::Arc<Vec<crate::app::key_handlers::ActionHint>>,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());

        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        render_header(frame, layout[0], state, context);
        render_queue_table(frame, layout[1], state, context);
        render_log_table(frame, layout[2], state, context);

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[3]);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &SyncStatusState, context: &RenderContext) {
    let header = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);
    let title = Line::from(vec![
        Span::styled("Sync Status", Style::default().fg(context.colors().accent)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", format_filter(state.filter())),
            Style::default().fg(context.colors().border),
        ),
    ])
    .centered();
    frame.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        header[0],
    );

    let snapshot = state.snapshot();
    let active = snapshot.active.as_ref().map(|job| match job.kind {
        crate::app::worker_controller::SyncJobKind::Pull => "Pull",
        crate::app::worker_controller::SyncJobKind::Push => "Push",
    });
    let summary = Line::from(vec![
        Span::styled("Queue", Style::default().fg(context.colors().accent)),
        Span::raw(format!(": {}  ", snapshot.queue_len)),
        Span::styled("Active", Style::default().fg(context.colors().accent)),
        Span::raw(format!(": {}  ", active.unwrap_or("Idle"))),
        Span::styled("Paused", Style::default().fg(context.colors().accent)),
        Span::raw(format!(
            ": {}  ",
            if snapshot.paused { "yes" } else { "no" }
        )),
        Span::styled("Errors", Style::default().fg(context.colors().accent)),
        Span::raw(format!(": {}  ", snapshot.error_count)),
        Span::styled("Last pull", Style::default().fg(context.colors().accent)),
        Span::raw(format!(": {}  ", format_time(snapshot.last_pull))),
        Span::styled("Last push", Style::default().fg(context.colors().accent)),
        Span::raw(format!(": {}", format_time(snapshot.last_push))),
    ])
    .centered();
    frame.render_widget(
        Paragraph::new(summary).alignment(Alignment::Center),
        header[1],
    );
}

fn render_queue_table(
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

fn render_log_table(
    frame: &mut Frame,
    area: Rect,
    state: &SyncStatusState,
    context: &RenderContext,
) {
    let mut rows = Vec::new();
    if state.sync_log().is_empty() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "No sync log entries",
                Style::default().fg(context.colors().border),
            )),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]));
    } else {
        for entry in state.sync_log() {
            rows.push(Row::new(vec![
                Cell::from(entry.created_at.clone()),
                Cell::from(entry.direction.clone()),
                Cell::from(entry.status.clone()),
                Cell::from(entry.error.clone().unwrap_or_default()),
            ]));
        }
    }

    let header = Row::new(["Time", "Direction", "Status", "Error"])
        .style(Style::default().fg(context.colors().accent));
    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from("Sync log").centered())
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

fn queue_row(
    state: &str,
    job: &crate::app::worker_controller::SyncJob,
    context: &RenderContext,
) -> Row<'static> {
    let kind = match job.kind {
        crate::app::worker_controller::SyncJobKind::Pull => "Pull",
        crate::app::worker_controller::SyncJobKind::Push => "Push",
    };
    let source = match job.source {
        crate::app::worker_controller::SyncSource::Manual => "Manual",
        crate::app::worker_controller::SyncSource::Button => "Button",
        crate::app::worker_controller::SyncSource::Startup => "Startup",
        crate::app::worker_controller::SyncSource::Interval => "Interval",
    };
    Row::new(vec![
        Cell::from(Span::styled(
            state.to_string(),
            Style::default().fg(context.colors().accent),
        )),
        Cell::from(kind),
        Cell::from(source),
        Cell::from(format_time(Some(job.created_at))),
        Cell::from(job.retries.to_string()),
        Cell::from(format_next_attempt(job.next_attempt_at)),
    ])
}

fn format_next_attempt(next_attempt: Option<std::time::Instant>) -> String {
    match next_attempt {
        None => "ready".to_string(),
        Some(instant) => {
            let now = std::time::Instant::now();
            if instant <= now {
                "ready".to_string()
            } else {
                let secs = instant.duration_since(now).as_secs();
                format!("in {secs}s")
            }
        }
    }
}

fn format_time(value: Option<std::time::SystemTime>) -> String {
    let Some(value) = value else {
        return "never".to_string();
    };
    let Ok(datetime) =
        time::OffsetDateTime::from(value).format(&time::format_description::well_known::Rfc3339)
    else {
        return "unknown".to_string();
    };
    datetime
}

fn format_filter(filter: SyncLogFilter) -> &'static str {
    match filter {
        SyncLogFilter::All => "All",
        SyncLogFilter::Pull => "Pull",
        SyncLogFilter::Push => "Push",
    }
}
