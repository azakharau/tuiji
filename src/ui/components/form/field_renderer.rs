use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::ui::{
    components::form::{FieldType, FormField},
    context::RenderContext,
};

mod feedback;
mod select;
mod text;

use feedback::{render_cursor, render_error};
use select::{render_multiselect_field, render_select_field};
use text::{render_text_field, render_textarea_field};

/// Renders a single form field based on its type.
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

    if is_selected {
        block = block.border_style(Style::default().fg(context.colors().accent));
    } else {
        block = block.border_style(Style::default().fg(context.colors().border));
    }

    let inner_area = block.inner(area);
    block.render(area, buf);

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

    match &field.field_type {
        FieldType::Text { is_password } => {
            render_text_field(field, inner_area, buf, *is_password, context)
        }
        FieldType::TextArea { .. } => render_textarea_field(field, inner_area, buf, context),
        FieldType::Select { options, expanded } => {
            render_select_field(field, options, *expanded, inner_area, buf, context)
        }
        FieldType::MultiSelect { options, expanded } => {
            render_multiselect_field(field, options, *expanded, inner_area, buf, context)
        }
    }

    if is_selected {
        render_cursor(field, area, buf, context);
    }

    if let Some(error) = &field.validation_state {
        render_error(error.message.as_str(), area, buf, context);
    }
}
