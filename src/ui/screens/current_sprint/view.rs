use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
};

use crate::{
    app::{key_handlers::ActionHint, state::Mode},
    ui::{components::bottom_bar::BottomBar, context::RenderContext},
};

use super::{
    kanban::KanbanView,
    state::{CurrentSprintState, SprintViewMode},
    table::TableView,
};

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
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        match state.view_mode() {
            SprintViewMode::Kanban => {
                KanbanView::draw(frame, layout[1], state, context);
            }
            SprintViewMode::Table => {
                TableView::draw(frame, layout[1], state, context);
            }
        }

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);
    }
}
