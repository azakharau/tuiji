use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
};

use crate::{
    ui::interaction::{ActionHint, Mode},
    ui::{
        components::{
            bottom_bar::BottomBar,
            list::{EmptyState, ListView},
        },
        context::RenderContext,
    },
};

use super::state::ConflictsState;

mod confirm_modal;
mod diff_table;
mod header;
mod issue_list;

use confirm_modal::render_confirm_modal;
use diff_table::{render_comment_diffs, render_issue_diff};
use header::render_header;
use issue_list::build_list_items;

pub struct ConflictsView;

impl ConflictsView {
    pub fn draw(
        frame: &mut Frame,
        state: &ConflictsState,
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

        render_header(frame, layout[0], context);

        if state.is_empty() {
            frame.render_widget(
                EmptyState::new("No conflicts", "You're up to date.", context),
                layout[1],
            );
        } else {
            let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(layout[1]);

            let list_items = build_list_items(state, context);
            let list = ListView {
                items: list_items.as_slice(),
                selected: state.selected_index(),
                context,
            };
            list.render(frame, body[0]);

            let right = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(body[1]);

            render_issue_diff(frame, state, right[0], context);
            render_comment_diffs(frame, state.comment_diffs(), right[1], context);
        }

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);

        if let Some(pending) = state.pending_resolve() {
            render_confirm_modal(frame, pending, context);
        }
    }
}
