use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

use crate::ui::{components::form::FormState, context::RenderContext};

use super::field_renderer::render_field;

pub struct FormView;

impl FormView {
    /// Calculate the Rect for the selected field (used for popup positioning)
    pub fn calculate_selected_field_rect(form: &FormState, area: Rect) -> Option<Rect> {
        Self::calculate_field_rect(form, area, form.selected_index())
    }

    pub fn calculate_field_rect(form: &FormState, area: Rect, field_index: usize) -> Option<Rect> {
        let fields = form.fields();
        if fields.is_empty() || area.height == 0 {
            return None;
        }

        let field_height: u16 = 3;
        let total = fields.len();
        let max_visible = (area.height / field_height).max(1) as usize;
        let visible = total.min(max_visible);
        let selected = field_index.min(total.saturating_sub(1));

        let mut offset = 0usize;
        if total > visible {
            let mut start = selected.saturating_sub(visible / 2);
            let max_start = total - visible;
            if start > max_start {
                start = max_start;
            }
            offset = start;
        }

        // Check if selected field is visible
        if selected < offset || selected >= offset + visible {
            return None;
        }

        let content_height = visible as u16 * field_height;
        let top = if total <= visible && area.height > content_height {
            area.y + (area.height - content_height) / 2
        } else {
            area.y
        };

        let side = (area.width / 20).max(2);
        let scrollbar_width = if total > visible { 1 } else { 0 };
        let field_width = area
            .width
            .saturating_sub(side.saturating_mul(2))
            .saturating_sub(scrollbar_width);
        let x = area.x + side;

        let i = selected - offset;
        let y = top + i as u16 * field_height;

        Some(Rect {
            x,
            y,
            width: field_width,
            height: field_height,
        })
    }

    pub fn render(
        form: &FormState,
        area: Rect,
        buf: &mut Buffer,
        context: &RenderContext,
        hide_content_for_field: Option<usize>,
    ) {
        let fields = form.fields();
        if fields.is_empty() || area.height == 0 {
            return;
        }

        let field_height: u16 = 3; // Minimum height per field
        let total = fields.len();
        let max_visible = (area.height / field_height).max(1) as usize;
        let visible = total.min(max_visible);
        let selected = form.selected_index().min(total.saturating_sub(1));

        let mut offset = 0usize;
        if total > visible {
            let mut start = selected.saturating_sub(visible / 2);
            let max_start = total - visible;
            if start > max_start {
                start = max_start;
            }
            offset = start;
        }

        let content_height = visible as u16 * field_height;
        let top = if total <= visible && area.height > content_height {
            area.y + (area.height - content_height) / 2
        } else {
            area.y
        };

        let side = (area.width / 20).max(2);
        let scrollbar_width = if total > visible { 1 } else { 0 };
        let field_width = area
            .width
            .saturating_sub(side.saturating_mul(2))
            .saturating_sub(scrollbar_width);
        let x = area.x + side;

        for i in 0..visible {
            let idx = offset + i;
            let field = &fields[idx];
            let y = top + i as u16 * field_height;
            let field_area = Rect {
                x,
                y,
                width: field_width,
                height: field_height,
            };

            let is_selected = selected == idx;
            let hide_content = hide_content_for_field == Some(idx);
            render_field(field, field_area, buf, is_selected, context, hide_content);
        }

        // Render scrollbar if needed
        if total > visible && area.width > 0 {
            let mut scrollbar_state = ScrollbarState::new(total).position(selected);
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(context.colors().accent));
            scrollbar.render(
                Rect {
                    x: area.x + area.width.saturating_sub(1),
                    y: area.y,
                    width: 1,
                    height: area.height,
                },
                buf,
                &mut scrollbar_state,
            );
        }
    }
}
