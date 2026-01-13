use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::ui::{components::form::FormField, context::RenderContext};

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

    /// Calculate popup area centered on screen
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

    /// Word-wrap a line to fit within the given width
    /// Breaks on word boundaries, keeping whole words together
    fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
        if line.is_empty() {
            return vec![String::new()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in line.split_whitespace() {
            let word_len = word.len();

            // If word itself is longer than max_width, break it
            if word_len > max_width {
                if !current_line.is_empty() {
                    result.push(current_line.clone());
                    current_line.clear();
                    current_width = 0;
                }

                // Break long word into chunks
                let mut remaining = word;
                while remaining.len() > max_width {
                    result.push(remaining[..max_width].to_string());
                    remaining = &remaining[max_width..];
                }
                if !remaining.is_empty() {
                    current_line = remaining.to_string();
                    current_width = remaining.len();
                }
                continue;
            }

            // Check if adding this word (with space) would exceed width
            let space_needed = if current_line.is_empty() { 0 } else { 1 };
            if current_width + space_needed + word_len > max_width {
                // Start new line
                result.push(current_line.clone());
                current_line = word.to_string();
                current_width = word_len;
            } else {
                // Add word to current line
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_len;
            }
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }

        if result.is_empty() {
            vec![String::new()]
        } else {
            result
        }
    }

    /// Calculate cursor position after word wrapping
    /// Takes the original cursor position in unwrapped text and returns (row, col) in wrapped text
    fn calculate_wrapped_cursor_position(
        text: &str,
        cursor_position: usize,
        max_width: usize,
    ) -> (usize, usize) {
        if text.is_empty() {
            return (0, 0);
        }

        // Track position in original text
        let mut original_pos = 0;
        let mut row = 0;

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line_len = 0;

        for (word_idx, word) in words.iter().enumerate() {
            let word_len = word.len();

            // Check if cursor is within this word
            if original_pos <= cursor_position && cursor_position <= original_pos + word_len {
                let col = current_line_len + (cursor_position - original_pos);
                return (row, col);
            }

            // Calculate if adding this word would fit on current line
            let space_needed = if current_line_len == 0 { 0 } else { 1 };

            if word_len > max_width {
                // Long word that will be broken
                let mut remaining_len = word_len;
                let mut word_start = original_pos;

                while remaining_len > 0 {
                    let chunk_len = remaining_len.min(max_width - current_line_len);

                    if word_start <= cursor_position && cursor_position <= word_start + chunk_len {
                        let col = current_line_len + (cursor_position - word_start);
                        return (row, col);
                    }

                    word_start += chunk_len;
                    remaining_len -= chunk_len;

                    if remaining_len > 0 {
                        row += 1;
                        current_line_len = 0;
                    } else {
                        current_line_len += chunk_len;
                    }
                }
            } else if current_line_len + space_needed + word_len > max_width {
                // Word doesn't fit, move to next line
                row += 1;
                current_line_len = word_len;
            } else {
                // Word fits on current line
                if current_line_len > 0 {
                    current_line_len += 1; // space
                }
                current_line_len += word_len;
            }

            // Move past this word and the space after it
            original_pos += word_len;
            if word_idx < words.len() - 1 {
                original_pos += 1; // space
            }
        }

        // Cursor is at or past the end
        (row, current_line_len)
    }
}

impl<'a> Widget for TextAreaPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the popup area first
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                // SAFETY: Coordinates (x, y) are guaranteed to be within buffer bounds
                // because they're derived from the area Rect provided by ratatui's layout system,
                // which ensures all areas fit within the terminal buffer dimensions.
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                }
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

        // Render text content with word wrapping
        let text = self.field.value.as_text().unwrap_or("");
        let max_width = inner_area.width as usize;

        // Apply word wrap to each line
        let wrapped_lines: Vec<String> = text
            .lines()
            .flat_map(|line| Self::wrap_line(line, max_width))
            .collect();

        // Convert to Line objects
        let lines: Vec<Line> = wrapped_lines
            .iter()
            .map(|line| Line::from(line.to_string()))
            .collect();

        // Calculate cursor position based on cursor state
        let (cursor_row, cursor_col) = match self.field.cursor {
            crate::ui::components::form::CursorState::TextArea { row, col } => (row, col),
            crate::ui::components::form::CursorState::Text { position } => {
                // For single-line Text fields with word-wrap, calculate wrapped position
                Self::calculate_wrapped_cursor_position(text, position, max_width)
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

        // Render cursor if enabled (but don't show if it would be out of bounds)
        if self.show_cursor {
            let display_row = cursor_row.saturating_sub(scroll_offset);
            if display_row < visible_rows && cursor_col < inner_area.width as usize {
                let cursor_x = inner_area.x + cursor_col as u16;
                let cursor_y = inner_area.y + display_row as u16;

                // Only render cursor if within bounds
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
