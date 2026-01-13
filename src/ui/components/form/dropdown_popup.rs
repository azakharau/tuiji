use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, List, ListItem, Widget},
};

use crate::ui::{
    components::form::{CursorState, FormField, SelectOption},
    context::RenderContext,
};

pub struct DropdownPopup<'a> {
    field: &'a FormField,
    options: &'a [SelectOption],
    context: &'a RenderContext,
}

impl<'a> DropdownPopup<'a> {
    pub fn new(
        field: &'a FormField,
        options: &'a [SelectOption],
        context: &'a RenderContext,
    ) -> Self {
        Self {
            field,
            options,
            context,
        }
    }

    /// Calculate popup position directly below the field
    pub fn calculate_area(field_rect: Rect, frame_area: Rect, options_count: usize) -> Rect {
        let popup_height = (options_count as u16 + 2).min(15); // Max 15 lines (with borders)
        let popup_width = field_rect.width.clamp(40, 60);

        // Align with field horizontally
        let popup_x = field_rect.x;

        // Position below field, or above if not enough space
        let below_y = field_rect.y + field_rect.height;
        let above_y = field_rect.y.saturating_sub(popup_height);

        let popup_y = if below_y + popup_height <= frame_area.y + frame_area.height {
            // Enough space below
            below_y
        } else if above_y >= frame_area.y {
            // Not enough space below, but enough above
            above_y
        } else {
            // Not enough space in either direction, put it below anyway
            below_y.min(frame_area.y + frame_area.height.saturating_sub(popup_height))
        };

        Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        }
    }
}

impl<'a> Widget for DropdownPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the popup area first
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.cell_mut((x, y)).unwrap().reset();
            }
        }

        // Background block with border
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.context.colors().accent))
            .style(
                Style::default()
                    .fg(self.context.colors().text)
                    .bg(self.context.colors().background),
            )
            .title(format!(" {} ", self.field.label));

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Get cursor index
        let cursor_index = match self.field.cursor {
            CursorState::Select { index } => index,
            CursorState::MultiSelect { index } => index,
            _ => 0,
        };

        // Calculate scroll offset if needed
        let visible_rows = inner_area.height as usize;
        let scroll_offset = if cursor_index >= visible_rows {
            cursor_index.saturating_sub(visible_rows / 2)
        } else {
            0
        };

        // Render list items
        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_rows)
            .map(|(idx, opt)| {
                let prefix = if opt.selected {
                    if matches!(self.field.cursor, CursorState::MultiSelect { .. }) {
                        "[✓] "
                    } else {
                        "• "
                    }
                } else if matches!(self.field.cursor, CursorState::MultiSelect { .. }) {
                    "[ ] "
                } else {
                    "  "
                };

                let style = if idx == cursor_index {
                    Style::default()
                        .fg(self.context.colors().accent)
                        .bg(self.context.colors().selection)
                } else {
                    Style::default().fg(self.context.colors().text)
                };

                ListItem::new(format!("{}{}", prefix, opt.label)).style(style)
            })
            .collect();

        List::new(items)
            .style(Style::default().bg(self.context.colors().background))
            .render(inner_area, buf);
    }
}
