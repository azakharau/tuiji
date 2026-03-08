use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::context::RenderContext;

use super::{
    super::state::SyncStatusState,
    formatting::{active_job_label, format_filter, format_time},
};

pub(super) fn render_header(
    frame: &mut Frame,
    area: Rect,
    state: &SyncStatusState,
    context: &RenderContext,
) {
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
    let summary = Line::from(vec![
        Span::styled("Queue", Style::default().fg(context.colors().accent)),
        Span::raw(format!(": {}  ", snapshot.queue_len)),
        Span::styled("Active", Style::default().fg(context.colors().accent)),
        Span::raw(format!(
            ": {}  ",
            active_job_label(snapshot.active.as_ref())
        )),
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
