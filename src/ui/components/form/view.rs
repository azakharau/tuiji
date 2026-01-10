use ratatui::{
    buffer::Buffer,
    layout::{Rect},
    style::Style,
    text::Line,
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};

use crate::ui::{components::form::FormState, context::RenderContext};

pub struct FormView;

impl FormView {
    pub fn render(form: &FormState, area: Rect, buf: &mut Buffer, context: &RenderContext) {
        let fields = form.fields();
        if fields.is_empty() || area.height == 0 {
            return;
        }

        let field_height: u16 = 3;
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
            let line = Rect {
                x,
                y,
                width: field_width,
                height: field_height,
            };
            let mut block = Block::bordered()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(
                    Style::default()
                        .fg(context.colors().text)
                        .bg(context.colors().background),
                )
                .border_style(Style::default().fg(context.colors().border))
                .title(Line::from(field.label));

            if selected == idx {
                block = block.border_style(Style::default().fg(context.colors().accent));
                render_cursor(field.cursor_position, line, buf, context);
            }
            let inner_area = block.inner(line);
            block.render(line, buf);
            Paragraph::new(field.masked_value())
                .style(
                    Style::default()
                        .fg(context.colors().text)
                        .bg(context.colors().background),
                )
                .render(inner_area, buf);
        }

        if total > visible && area.width > 0 {
            let mut scrollbar_state = ScrollbarState::new(total)
                .position(selected);
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

fn render_cursor(cursor_position: usize, line: Rect, buf: &mut Buffer, context: &RenderContext) {
    let cursor_x = line.x + 1 + cursor_position as u16;
    let cursor_y = line.y + 1;
    let rect = Rect {
        x: cursor_x,
        y: cursor_y,
        width: 1,
        height: 1,
    };
    let cursor_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(context.colors().accent));
    cursor_block.render(rect, buf);
}
