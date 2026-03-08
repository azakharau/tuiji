use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::ui::context::RenderContext;

use super::super::state::SyncStatusState;

pub(super) fn render_log_table(
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
