use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear},
};

use crate::{
    ui::interaction::{ActionHint, Mode},
    ui::{
        components::{bottom_bar::BottomBar, form::FormView},
        context::RenderContext,
        overlays::ErrorModal,
    },
};

use super::state::SettingsThemeFormState;

pub struct SettingsThemeFormView;

impl SettingsThemeFormView {
    pub fn draw(
        frame: &mut Frame,
        state: &SettingsThemeFormState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        let area = crate::ui::layout::modal_dialog_area(frame.area());
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(context.colors().border))
            .style(
                ratatui::style::Style::default()
                    .bg(context.colors().background)
                    .fg(context.colors().text),
            )
            .title(Line::from("Custom Theme").centered());

        let inner = block.inner(area);
        let [form_area, bar_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        let buffer = frame.buffer_mut();
        FormView::render(state.form(), form_area, buffer, context, None);

        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, bar_area);
        if let Some(err) = state.error() {
            frame.render_widget(ErrorModal::new(err, context), frame.area());
        }
    }
}
