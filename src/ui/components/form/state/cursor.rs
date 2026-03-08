use super::super::field_type::{CursorState, FieldType, FieldValue};
use super::text_ops::{
    next_char_boundary, prev_char_boundary, word_backward, word_end, word_forward,
};
use super::*;

impl FormState {
    pub fn move_cursor_left(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.field_type, &mut field.cursor, &field.value) {
                (FieldType::Text { .. }, CursorState::Text { position }, _) => {
                    if let FieldValue::Text(value) = &field.value {
                        for _ in 0..repeat {
                            *position = prev_char_boundary(value, *position);
                        }
                    }
                }
                (
                    FieldType::TextArea { .. },
                    CursorState::TextArea { row, col },
                    FieldValue::Text(s),
                ) => {
                    let lines: Vec<&str> = s.lines().collect();
                    if lines.is_empty() {
                        *row = 0;
                        *col = 0;
                        return;
                    }

                    for _ in 0..repeat {
                        if *col > 0 {
                            *col = prev_char_boundary(lines.get(*row).copied().unwrap_or(""), *col);
                        } else if *row > 0 {
                            *row -= 1;
                            *col = lines.get(*row).map(|line| line.len()).unwrap_or(0);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn move_cursor_right(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.field_type, &mut field.cursor, &field.value) {
                (FieldType::Text { .. }, CursorState::Text { position }, FieldValue::Text(s)) => {
                    for _ in 0..repeat {
                        *position = next_char_boundary(s, *position);
                    }
                }
                (
                    FieldType::TextArea { .. },
                    CursorState::TextArea { row, col },
                    FieldValue::Text(s),
                ) => {
                    let lines: Vec<&str> = s.lines().collect();
                    if lines.is_empty() {
                        *row = 0;
                        *col = 0;
                        return;
                    }

                    for _ in 0..repeat {
                        let current_line = lines.get(*row).unwrap_or(&"");
                        if *col < current_line.len() {
                            *col = next_char_boundary(current_line, *col);
                        } else if *row + 1 < lines.len() {
                            *row += 1;
                            *col = 0;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn move_cursor_up(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut()
            && let (
                FieldType::TextArea { .. },
                CursorState::TextArea { row, col },
                FieldValue::Text(s),
            ) = (&field.field_type, &mut field.cursor, &field.value)
        {
            let lines: Vec<&str> = s.lines().collect();
            if lines.is_empty() {
                return;
            }

            *row = row.saturating_sub(repeat);
            let current_line = lines.get(*row).unwrap_or(&"");
            *col = (*col).min(current_line.len());
        }
    }

    pub fn move_cursor_down(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut()
            && let (
                FieldType::TextArea { .. },
                CursorState::TextArea { row, col },
                FieldValue::Text(s),
            ) = (&field.field_type, &mut field.cursor, &field.value)
        {
            let lines: Vec<&str> = s.lines().collect();
            if lines.is_empty() {
                return;
            }

            *row = (*row + repeat).min(lines.len().saturating_sub(1));
            let current_line = lines.get(*row).unwrap_or(&"");
            *col = (*col).min(current_line.len());
        }
    }

    pub fn move_cursor_line_start(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match &mut field.cursor {
                CursorState::Text { position } => *position = 0,
                CursorState::TextArea { col, .. } => *col = 0,
                _ => {}
            }
        }
    }

    pub fn move_cursor_line_end(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.field_type, &mut field.cursor, &field.value) {
                (FieldType::Text { .. }, CursorState::Text { position }, FieldValue::Text(s)) => {
                    *position = s.len();
                }
                (
                    FieldType::TextArea { .. },
                    CursorState::TextArea { row, col },
                    FieldValue::Text(s),
                ) => {
                    let lines: Vec<&str> = s.lines().collect();
                    *col = lines.get(*row).map(|line| line.len()).unwrap_or(0);
                }
                _ => {}
            }
        }
    }

    pub fn move_word_right(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut()
            && let (CursorState::Text { position }, FieldValue::Text(value)) =
                (&mut field.cursor, &field.value)
        {
            for _ in 0..repeat {
                *position = word_forward(value, *position);
            }
        }
    }

    pub fn move_word_end(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut()
            && let (CursorState::Text { position }, FieldValue::Text(value)) =
                (&mut field.cursor, &field.value)
        {
            for _ in 0..repeat {
                *position = word_end(value, *position);
            }
        }
    }

    pub fn move_word_left(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut()
            && let (CursorState::Text { position }, FieldValue::Text(value)) =
                (&mut field.cursor, &field.value)
        {
            for _ in 0..repeat {
                *position = word_backward(value, *position);
            }
        }
    }
}
