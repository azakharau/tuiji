use super::*;
use crate::ui::components::form::SelectOption;

pub(super) fn render_select_field(
    field: &FormField,
    options: &[SelectOption],
    expanded: bool,
    area: Rect,
    buf: &mut Buffer,
    context: &RenderContext,
) {
    let selected_label = field
        .value
        .as_single()
        .and_then(|value| options.iter().find(|option| option.value == value))
        .map(|option| option.label.as_str())
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

pub(super) fn render_multiselect_field(
    field: &FormField,
    _options: &[SelectOption],
    expanded: bool,
    area: Rect,
    buf: &mut Buffer,
    context: &RenderContext,
) {
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
