use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders},
};

use crate::{
    ui::interaction::{ActionHint, Mode},
    ui::{
        components::{
            bottom_bar::BottomBar,
            form::{DropdownPopup, FieldType, FormView, TextAreaPopup},
        },
        context::RenderContext,
        overlays::ErrorModal,
    },
};

use super::state::IssueFormState;

pub struct IssueFormView;

impl IssueFormView {
    pub fn draw(
        frame: &mut Frame,
        state: &IssueFormState,
        mode: Mode,
        actions: &Arc<Vec<ActionHint>>,
        context: &RenderContext,
    ) {
        // Full screen background
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());

        // Main layout: title bar + content + bottom bar
        let layout = Layout::vertical([
            Constraint::Length(3), // Title bar
            Constraint::Fill(1),   // Form content
            Constraint::Length(1), // Bottom bar
        ])
        .split(frame.area());

        // Title bar with border
        let title_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .style(base_style)
            .border_style(Style::default().fg(context.colors().border))
            .title(Line::from(state.title()).centered());
        frame.render_widget(title_block, layout[0]);

        // Form content area with side borders
        let content_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(context.colors().border))
            .style(base_style);

        let form_area = content_block.inner(layout[1]);
        frame.render_widget(content_block, layout[1]);

        // Hide field content if text popup is open
        let hide_content_for = if state.is_text_popup_open() {
            Some(state.form().selected_index())
        } else {
            None
        };

        let buffer = frame.buffer_mut();
        FormView::render(state.form(), form_area, buffer, context, hide_content_for);

        // Bottom bar
        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);

        // Render text popup overlay if open
        if state.is_text_popup_open()
            && let Some(field) = state.form().selected_field()
            && matches!(
                field.field_type,
                FieldType::Text { .. } | FieldType::TextArea { .. }
            )
        {
            let min_height = if matches!(field.field_type, FieldType::TextArea { .. }) {
                10
            } else {
                5
            };
            let popup_area = TextAreaPopup::calculate_area(frame.area(), min_height);
            // Show cursor in popup for editing
            let popup = TextAreaPopup::new(field, context, true);
            frame.render_widget(popup, popup_area);
        }

        // Render dropdown popup overlay if a select/multiselect field is expanded
        if let Some(field) = state.form().selected_field() {
            match &field.field_type {
                FieldType::Select { options, expanded } if *expanded => {
                    if let Some(field_rect) =
                        FormView::calculate_selected_field_rect(state.form(), form_area)
                    {
                        let popup_area =
                            DropdownPopup::calculate_area(field_rect, frame.area(), options.len());
                        let popup = DropdownPopup::new(field, options, context);
                        frame.render_widget(popup, popup_area);
                    }
                }
                FieldType::MultiSelect { options, expanded } if *expanded => {
                    if let Some(field_rect) =
                        FormView::calculate_selected_field_rect(state.form(), form_area)
                    {
                        let popup_area =
                            DropdownPopup::calculate_area(field_rect, frame.area(), options.len());
                        let popup = DropdownPopup::new(field, options, context);
                        frame.render_widget(popup, popup_area);
                    }
                }
                _ => {}
            }
        }

        // Error modal overlay (if present)
        if let Some(err) = state.error() {
            frame.render_widget(ErrorModal::new(err, context), frame.area());
        }
    }
}
