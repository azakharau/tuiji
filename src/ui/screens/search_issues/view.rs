use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use crate::{
    app::{key_handlers::ActionHint, state::Mode},
    ui::{
        components::{
            bottom_bar::BottomBar,
            list::{EmptyState, TableView},
        },
        context::RenderContext,
    },
};

use super::state::SearchIssuesState;

pub struct SearchIssuesView;

impl SearchIssuesView {
    pub fn draw(
        frame: &mut Frame,
        state: &SearchIssuesState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(
            ratatui::widgets::Block::default().style(base_style),
            frame.area(),
        );
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let title = Paragraph::new(Line::from("Search Issues").centered())
            .style(Style::default().fg(context.colors().accent))
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        if state.is_empty() {
            frame.render_widget(
                EmptyState::new("No issues", "Sync to load issues.", context),
                layout[1],
            );
        } else {
            let table = TableView {
                rows: state.rows(),
                selected: state.selected_index(),
                context,
            };
            table.render(frame, layout[1]);
        }

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);
    }
}
