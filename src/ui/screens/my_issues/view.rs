use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::ui::{
    components::{
        bottom_bar::BottomBar,
        list::{EmptyState, TableView},
    },
    context::RenderContext,
    interaction::{ActionHint, Mode},
};

use super::state::MyIssuesState;

pub struct MyIssuesView;

impl MyIssuesView {
    pub fn draw(
        frame: &mut Frame,
        state: &mut MyIssuesState,
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

        let title = Paragraph::new(Line::from("My Issues").centered())
            .style(Style::default().fg(context.colors().accent))
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        state
            .table_mut()
            .set_rows_visible(layout[1].height.saturating_sub(1) as usize);
        if state.table().is_empty() {
            let (title, message) = state
                .error()
                .map(|error| ("Unable to load My Issues", error))
                .unwrap_or((
                    "No assigned issues",
                    "No unresolved issues are assigned to you.",
                ));
            frame.render_widget(EmptyState::new(title, message, context), layout[1]);
        } else {
            TableView {
                rows: state.table().visible_rows(),
                selected: state.table().visible_selected_index(),
                context,
            }
            .render(frame, layout[1]);
        }

        frame.render_widget(BottomBar::new(mode, actions.clone(), context), layout[2]);
    }
}
