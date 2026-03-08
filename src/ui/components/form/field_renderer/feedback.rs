use super::*;
use crate::ui::components::form::CursorState;
use ratatui::text::Span;

pub(super) fn render_cursor(
    field: &FormField,
    area: Rect,
    buf: &mut Buffer,
    context: &RenderContext,
) {
    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);
    if inner_width == 0 || inner_height == 0 {
        return;
    }

    match (&field.field_type, &field.cursor) {
        (FieldType::Text { .. }, CursorState::Text { position }) => {
            let cursor_x = area.x + 1 + (*position as u16).min(inner_width.saturating_sub(1));
            let cursor_y = area.y + 1;
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
        (FieldType::TextArea { .. }, CursorState::TextArea { row, col }) => {
            let visible_rows = inner_height as usize;
            let scroll_offset = if *row >= visible_rows {
                row.saturating_sub(visible_rows / 2)
            } else {
                0
            };
            let display_row = row.saturating_sub(scroll_offset);
            if display_row >= visible_rows {
                return;
            }

            let cursor_x = area.x + 1 + (*col as u16).min(inner_width.saturating_sub(1));
            let cursor_y = area.y + 1 + (display_row as u16).min(inner_height.saturating_sub(1));
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
        _ => {}
    }
}

pub(super) fn render_error(message: &str, area: Rect, buf: &mut Buffer, context: &RenderContext) {
    if area.height < 3 {
        return;
    }
    let error_area = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(1),
        width: area.width.saturating_sub(4),
        height: 1,
    };
    let error_text = Span::styled(
        format!("⚠ {}", message),
        Style::default().fg(context.colors().error),
    );
    Paragraph::new(Line::from(error_text)).render(error_area, buf);
}
