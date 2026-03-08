use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
};

use crate::{
    ui::interaction::{ActionHint, Mode},
    ui::{components::bottom_bar::BottomBar, context::RenderContext},
};

use super::{detail::render_issue_detail_modal, state::CurrentSprintState, table::TableView};

pub struct CurrentSprintView;

impl CurrentSprintView {
    pub fn draw(
        frame: &mut Frame,
        state: &CurrentSprintState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        TableView::draw(frame, layout[1], state, context);

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);

        if state.detail_open()
            && let Some(issue) = state.selected_issue()
        {
            render_issue_detail_modal(frame, issue, context);
        }
    }
}
