use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::ui::{components::form::FormField, context::RenderContext};

mod wrap;

use wrap::{calculate_wrapped_cursor_position, wrap_line};

pub struct TextAreaPopup<'a> {
    field: &'a FormField,
    context: &'a RenderContext,
    show_cursor: bool,
}

impl<'a> TextAreaPopup<'a> {
    pub fn new(field: &'a FormField, context: &'a RenderContext, show_cursor: bool) -> Self {
        Self {
            field,
            context,
            show_cursor,
        }
    }

    /// Calculate popup area centered on screen.
    pub fn calculate_area(frame_area: Rect, min_height: u16) -> Rect {
        let popup_width = (frame_area.width * 3 / 4).min(80);
        let popup_height = (frame_area.height * 2 / 3).max(min_height + 2);

        let popup_x = frame_area
            .x
            .saturating_add((frame_area.width.saturating_sub(popup_width)) / 2);
        let popup_y = frame_area
            .y
            .saturating_add((frame_area.height.saturating_sub(popup_height)) / 2);

        Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        }
    }
}

impl<'a> Widget for TextAreaPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.cell_mut((x, y))
                    .expect("cell within popup area")
                    .reset();
            }
        }

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

        let text = self.field.value.as_text().unwrap_or("");
        let max_width = inner_area.width as usize;

        let wrapped_lines: Vec<String> = text
            .lines()
            .flat_map(|line| wrap_line(line, max_width))
            .collect();

        let lines: Vec<Line> = wrapped_lines
            .iter()
            .map(|line| Line::from(line.to_string()))
            .collect();

        let (cursor_row, cursor_col) = match self.field.cursor {
            crate::ui::components::form::CursorState::TextArea { row, col } => (row, col),
            crate::ui::components::form::CursorState::Text { position } => {
                calculate_wrapped_cursor_position(text, position, max_width)
            }
            _ => (0, 0),
        };

        let visible_rows = inner_area.height as usize;
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
                .fg(self.context.colors().text)
                .bg(self.context.colors().background),
        );

        paragraph.render(inner_area, buf);

        if self.show_cursor {
            let display_row = cursor_row.saturating_sub(scroll_offset);
            if display_row < visible_rows && cursor_col < inner_area.width as usize {
                let cursor_x = inner_area.x + cursor_col as u16;
                let cursor_y = inner_area.y + display_row as u16;

                if cursor_x < inner_area.x + inner_area.width
                    && cursor_y < inner_area.y + inner_area.height
                {
                    let rect = Rect {
                        x: cursor_x,
                        y: cursor_y,
                        width: 1,
                        height: 1,
                    };
                    let cursor_block =
                        Block::default().style(Style::default().bg(self.context.colors().accent));
                    cursor_block.render(rect, buf);
                }
            }
        }
    }
}
