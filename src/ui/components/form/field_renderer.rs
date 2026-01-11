use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::ui::{
    components::form::{CursorState, FieldType, FormField},
    context::RenderContext,
};

/// Renders a single form field based on its type
pub fn render_field(
    field: &FormField,
    area: Rect,
    buf: &mut Buffer,
    is_selected: bool,
    context: &RenderContext,
    hide_content: bool,
) {
    let mut block = Block::bordered()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(
            Style::default()
                .fg(context.colors().text)
                .bg(context.colors().background),
        )
        .title(Line::from(format!(
            "{} {}",
            field.label,
            if field.required { "*" } else { "" }
        )));

    // Highlight border if selected
    if is_selected {
        block = block.border_style(Style::default().fg(context.colors().accent));
    } else {
        block = block.border_style(Style::default().fg(context.colors().border));
    }

    let inner_area = block.inner(area);
    block.render(area, buf);

    // If content should be hidden (e.g., popup is open), show placeholder
    if hide_content {
        let placeholder = match &field.field_type {
            FieldType::Text { .. } | FieldType::TextArea { .. } => "(editing in popup...)",
            _ => "",
        };
        Paragraph::new(placeholder)
            .style(
                Style::default()
                    .fg(context.colors().border)
                    .bg(context.colors().background),
            )
            .render(inner_area, buf);
        return;
    }

    // Render field content based on type
    match &field.field_type {
        FieldType::Text { is_password } => {
            render_text_field(field, inner_area, buf, is_selected, *is_password, context)
        }
        FieldType::TextArea { .. } => {
            render_textarea_field(field, inner_area, buf, is_selected, context)
        }
        FieldType::Select { options, expanded } => render_select_field(
            field,
            options,
            *expanded,
            inner_area,
            buf,
            is_selected,
            context,
        ),
        FieldType::MultiSelect { options, expanded } => render_multiselect_field(
            field,
            options,
            *expanded,
            inner_area,
            buf,
            is_selected,
            context,
        ),
    }

    // Render cursor for selected field
    if is_selected {
        render_cursor(field, area, buf, context);
    }

    // Render validation error if present
    if let Some(error) = &field.validation_state {
        render_error(error.message.as_str(), area, buf, context);
    }
}

/// Renders a single-line text field
fn render_text_field(
    field: &FormField,
    area: Rect,
    buf: &mut Buffer,
    _is_selected: bool,
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

/// Renders a multi-line textarea field
fn render_textarea_field(
    field: &FormField,
    area: Rect,
    buf: &mut Buffer,
    _is_selected: bool,
    context: &RenderContext,
) {
    let text = field.value.as_text().unwrap_or("");

    // Split text into lines for display
    let lines: Vec<Line> = text
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect();

    // Calculate scroll offset if needed
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

/// Renders a select dropdown field
fn render_select_field(
    field: &FormField,
    _options: &[crate::ui::components::form::SelectOption],
    expanded: bool,
    area: Rect,
    buf: &mut Buffer,
    _is_selected: bool,
    context: &RenderContext,
) {
    // Always show selected value (popup is rendered separately)
    let selected_label = field
        .value
        .as_single()
        .and_then(|val| _options.iter().find(|opt| opt.value == val))
        .map(|opt| opt.label.as_str())
        .unwrap_or("Select...");

    let arrow = if expanded { " ▲" } else { " ▼" };

    Paragraph::new(format!("{}{}", selected_label, arrow))
        .style(
            Style::default()
                .fg(context.colors().text)
                .bg(context.colors().background),
        )
        .render(area, buf);
}

/// Renders a multi-select field with checkboxes
fn render_multiselect_field(
    field: &FormField,
    _options: &[crate::ui::components::form::SelectOption],
    expanded: bool,
    area: Rect,
    buf: &mut Buffer,
    _is_selected: bool,
    context: &RenderContext,
) {
    // Always show selected count (popup is rendered separately)
    let selected = field.value.as_multiple().unwrap_or(&[]);
    let count = selected.len();
    let arrow = if expanded { " ▲" } else { " ▼" };
    let text = if count == 0 {
        format!("Select items...{}", arrow)
    } else {
        format!("{} item(s) selected{}", count, arrow)
    };

    Paragraph::new(text)
        .style(
            Style::default()
                .fg(context.colors().text)
                .bg(context.colors().background),
        )
        .render(area, buf);
}

/// Renders cursor for the selected field
fn render_cursor(field: &FormField, area: Rect, buf: &mut Buffer, context: &RenderContext) {
    match (&field.field_type, &field.cursor) {
        (FieldType::Text { .. }, CursorState::Text { position }) => {
            let cursor_x = area.x + 1 + *position as u16;
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
            let cursor_x = area.x + 1 + *col as u16;
            let cursor_y = area.y + 1 + *row as u16;
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
        _ => {
            // Select/MultiSelect don't need custom cursor rendering
        }
    }
}

/// Renders validation error below the field
fn render_error(message: &str, area: Rect, buf: &mut Buffer, context: &RenderContext) {
    if area.height < 3 {
        return; // Not enough space
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
