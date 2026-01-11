use ratatui::Frame;

use crate::ui::{components::kanban_board::KanbanBoard, context::RenderContext};

use super::state::CurrentSprintState;

pub struct KanbanView;

impl KanbanView {
    pub fn draw(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &CurrentSprintState,
        context: &RenderContext,
    ) {
        let kanban_board = KanbanBoard::new(
            1,
            "Current Sprint".to_string(),
            state.issues().clone(),
            state.board_cfg(),
            context,
            state.selected_col(),
            state.selected_row(),
            state.scroll_offset(),
            state.rows_visible(),
        );
        frame.render_widget(kanban_board, area);
    }
}
