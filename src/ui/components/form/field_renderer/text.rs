use super::*;
use crate::ui::components::form::CursorState;

pub(super) fn render_text_field(
    field: &FormField,
    area: Rect,
    buf: &mut Buffer,
    is_password: bool,
    context: &RenderContext,
) {
    let display_value = if is_password {
        if let Some(text) = field.value.as_text() {
            "*".repeat(text.len())
        } else {
            String::new()
        }
    } else {
        field.display_value()
    };

    Paragraph::new(display_value)
        .style(
            Style::default()
                .fg(context.colors().text)
                .bg(context.colors().background),
        )
        .render(area, buf);
}

pub(super) fn render_textarea_field(
    field: &FormField,
    area: Rect,
    buf: &mut Buffer,
    context: &RenderContext,
) {
    let text = field.value.as_text().unwrap_or("");

    let lines: Vec<Line> = text
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect();

    let cursor_row = if let CursorState::TextArea { row, .. } = field.cursor {
        row
    } else {
        0
    };

    let visible_rows = area.height as usize;

    let scroll_offset = if cursor_row >= visible_rows {
        cursor_row.saturating_sub(visible_rows / 2)
    } else {
        0
    };

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_rows)
        .collect();

    let paragraph = Paragraph::new(visible_lines).style(
        Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background),
    );

    paragraph.render(area, buf);
}
