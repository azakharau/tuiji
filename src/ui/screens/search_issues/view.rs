use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::{
    components::{
        bottom_bar::BottomBar,
        list::{EmptyState, TableView},
    },
    context::RenderContext,
    interaction::{ActionHint, Mode},
};

use super::state::SearchIssuesState;

pub struct SearchIssuesView;

impl SearchIssuesView {
    pub fn draw(
        frame: &mut Frame,
        state: &mut SearchIssuesState,
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
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let title = Paragraph::new(Line::from("Search Issues").centered())
            .style(Style::default().fg(context.colors().accent))
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        let active_query = if state.active_query().is_empty() {
            "Results: no query run yet".to_string()
        } else {
            format!("Results for: {}", state.active_query())
        };
        frame.render_widget(
            Paragraph::new(active_query).style(Style::default().fg(context.colors().info)),
            layout[1],
        );

        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("JQL  / to edit, Enter to run")
            .border_style(Style::default().fg(if mode == Mode::Insert {
                context.colors().accent
            } else {
                context.colors().border
            }));
        let input_inner = input_block.inner(layout[2]);
        let input_width = input_inner.width as usize;
        let scroll = state.input().visual_scroll(input_width);
        frame.render_widget(
            Paragraph::new(state.input().value())
                .style(base_style)
                .scroll((0, scroll as u16))
                .block(input_block),
            layout[2],
        );
        if mode == Mode::Insert && input_inner.width > 0 {
            let cursor = state.input().visual_cursor().saturating_sub(scroll);
            frame.set_cursor_position((
                input_inner.x + cursor.min(input_inner.width.saturating_sub(1) as usize) as u16,
                input_inner.y,
            ));
        }

        state
            .table_mut()
            .set_rows_visible(layout[3].height.saturating_sub(1) as usize);
        if state.table().is_empty() {
            let (title, message) = if let Some(error) = state.error() {
                ("Unable to search Jira", error)
            } else if state.active_query().is_empty() {
                (
                    "Search Jira",
                    "Press /, enter a JQL query, then press Enter.",
                )
            } else {
                (
                    "No matching issues",
                    "Jira returned no issues for this query.",
                )
            };
            frame.render_widget(EmptyState::new(title, message, context), layout[3]);
        } else {
            TableView {
                rows: state.table().visible_rows(),
                selected: state.table().visible_selected_index(),
                context,
            }
            .render(frame, layout[3]);
        }

        frame.render_widget(BottomBar::new(mode, actions.clone(), context), layout[4]);
    }
}
