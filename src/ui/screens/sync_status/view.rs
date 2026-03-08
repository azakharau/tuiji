use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
};

use crate::{
    ui::interaction::Mode,
    ui::{components::bottom_bar::BottomBar, context::RenderContext},
};

use super::state::SyncStatusState;

mod formatting;
mod header;
mod log_table;
mod queue_table;

use header::render_header;
use log_table::render_log_table;
use queue_table::render_queue_table;

pub struct SyncStatusView;

impl SyncStatusView {
    pub fn draw(
        frame: &mut Frame,
        state: &SyncStatusState,
        mode: Mode,
        actions: &std::sync::Arc<Vec<crate::ui::interaction::ActionHint>>,
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
