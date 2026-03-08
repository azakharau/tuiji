use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::Text,
    widgets::{Paragraph, Wrap},
};

use crate::{
    ui::{
        components::{
            bottom_bar::BottomBar,
            list::{EmptyState, ListView},
        },
        context::RenderContext,
    },
    ui::{
        interaction::{ActionHint, Mode},
        layout::modal_dialog_area,
    },
};

use super::state::ProfilesState;

pub struct ProfilesView;

const LIST_SIDE_PADDING: u16 = 15;

impl ProfilesView {
    pub fn draw(
        frame: &mut Frame,
        state: &ProfilesState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        let area = modal_dialog_area(frame.area());
        let block = ratatui::widgets::Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(ratatui::style::Style::default().fg(context.colors().border))
            .style(
                ratatui::style::Style::default()
                    .fg(context.colors().text)
                    .bg(context.colors().background),
            )
            .title(ratatui::text::Line::from("Profiles").centered());
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(&block, area);
        let inner = block.inner(area);
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(inner);
        let list_area = Layout::horizontal([
            Constraint::Length(LIST_SIDE_PADDING),
            Constraint::Fill(1),
            Constraint::Length(LIST_SIDE_PADDING),
        ])
        .split(layout[1])[1];
        let text = Paragraph::new(Text::from(state.message()))
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        frame.render_widget(text, layout[0]);
        let items = state.list_items();
        if items.is_empty() {
            frame.render_widget(
                EmptyState::new("Profiles", state.message(), context),
                layout[1],
            );
        } else {
            let list_view = ListView {
                items,
                selected: state.selected_index(),
                context,
            };
            list_view.render(frame, list_area);
        }
        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[3]);
    }
}
