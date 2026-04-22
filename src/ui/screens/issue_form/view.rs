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

use super::state::{IssueFormState, IssueFormSurface};

enum ActiveOverlay<'a> {
    TextPopup {
        field: &'a crate::ui::components::form::FormField,
        min_height: u16,
    },
    Dropdown {
        field_index: usize,
        field: &'a crate::ui::components::form::FormField,
        options: &'a [crate::ui::components::form::SelectOption],
    },
}

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
        let buffer = frame.buffer_mut();
        FormView::render(
            state.form(),
            form_area,
            buffer,
            context,
            state.hide_form_content_for(),
        );

        // Bottom bar
        let bottom_bar = BottomBar::new(mode, actions.clone(), context);
        frame.render_widget(bottom_bar, layout[2]);

        Self::render_active_overlay(frame, state, form_area, context);

        // Error modal overlay (if present)
        if let Some(err) = state.error() {
            frame.render_widget(ErrorModal::new(err, context), frame.area());
        }
    }

    fn render_active_overlay(
        frame: &mut Frame,
        state: &IssueFormState,
        form_area: ratatui::layout::Rect,
        context: &RenderContext,
    ) {
        let Some(overlay) = Self::active_overlay(state) else {
            return;
        };

        match overlay {
            ActiveOverlay::TextPopup { field, min_height } => {
                let popup_area = TextAreaPopup::calculate_area(frame.area(), min_height);
                frame.render_widget(TextAreaPopup::new(field, context, true), popup_area);
            }
            ActiveOverlay::Dropdown {
                field_index,
                field,
                options,
            } => {
                if let Some(field_rect) =
                    FormView::calculate_field_rect(state.form(), form_area, field_index)
                {
                    let popup_area =
                        DropdownPopup::calculate_area(field_rect, frame.area(), options.len());
                    frame.render_widget(DropdownPopup::new(field, options, context), popup_area);
                }
            }
        }
    }

    fn active_overlay(state: &IssueFormState) -> Option<ActiveOverlay<'_>> {
        let field_index = state
            .active_overlay_field_index()
            .unwrap_or_else(|| state.form().selected_index());
        let field = state.form().fields().get(field_index)?;

        match state.active_surface() {
            IssueFormSurface::TextPopup { .. } => {
                if !matches!(
                    field.field_type,
                    FieldType::Text { .. } | FieldType::TextArea { .. }
                ) {
                    return None;
                }

                let min_height = if matches!(field.field_type, FieldType::TextArea { .. }) {
                    10
                } else {
                    5
                };

                Some(ActiveOverlay::TextPopup { field, min_height })
            }
            IssueFormSurface::Dropdown { .. } => match &field.field_type {
                FieldType::Select { options, .. } | FieldType::MultiSelect { options, .. } => {
                    Some(ActiveOverlay::Dropdown {
                        field_index,
                        field,
                        options,
                    })
                }
                _ => None,
            },
            IssueFormSurface::Form => None,
        }
    }
}
