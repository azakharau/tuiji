use super::super::field_type::{CursorState, FieldValue};
use super::text_ops::{line_len_at_row, row_col_to_position};
use super::*;

impl FormState {
    pub fn insert_char(&mut self, ch: char) {
        if let Some(field) = self.selected_field_mut() {
            match (&mut field.cursor, &mut field.value) {
                (CursorState::Text { position }, FieldValue::Text(s)) => {
                    s.insert(*position, ch);
                    *position += 1;
                }
                (CursorState::TextArea { row, col }, FieldValue::Text(s)) => {
                    let pos = row_col_to_position(s, *row, *col);
                    s.insert(pos, ch);

                    if ch == '\n' {
                        *row += 1;
                        *col = 0;
                    } else {
                        *col += 1;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn backspace(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match (&mut field.cursor, &mut field.value) {
                (CursorState::Text { position }, FieldValue::Text(s)) => {
                    if *position > 0 {
                        *position -= 1;
                        s.remove(*position);
                    }
                }
                (CursorState::TextArea { row, col }, FieldValue::Text(s)) => {
                    if *col > 0 {
                        let pos = row_col_to_position(s, *row, *col);
                        s.remove(pos - 1);
                        *col -= 1;
                    } else if *row > 0 {
                        let lines: Vec<&str> = s.lines().collect();
                        let prev_line_len = lines.get(*row - 1).map(|line| line.len()).unwrap_or(0);

                        let pos = row_col_to_position(s, *row, *col);
                        if pos > 0 {
                            s.remove(pos - 1);
                            *row -= 1;
                            *col = prev_line_len;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn delete(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.cursor, &mut field.value) {
                (CursorState::Text { position }, FieldValue::Text(s)) => {
                    if *position < s.len() {
                        s.remove(*position);
                    }
                }
                (CursorState::TextArea { row, col }, FieldValue::Text(s)) => {
                    let pos = row_col_to_position(s, *row, *col);
                    if pos < s.len() {
                        s.remove(pos);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn enter_insert_before(&mut self) {
        self.move_cursor_line_start();
    }

    pub fn enter_insert_after(&mut self) {
        if let Some(field) = self.selected_field_mut()
            && let (CursorState::Text { position }, FieldValue::Text(s)) =
                (&field.cursor, &field.value)
        {
            let pos = *position;
            let len = s.len();
            if pos < len
                && let CursorState::Text { position } = &mut field.cursor
            {
                *position = pos + 1;
            }
        }
    }

    pub fn enter_insert_line_start(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match &mut field.cursor {
                CursorState::Text { position } => *position = 0,
                CursorState::TextArea { col, .. } => *col = 0,
                _ => {}
            }
        }
    }

    pub fn enter_insert_line_end(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.cursor, &field.value) {
                (CursorState::Text { .. }, FieldValue::Text(s)) => {
                    if let CursorState::Text { position } = &mut field.cursor {
                        *position = s.len();
                    }
                }
                (CursorState::TextArea { .. }, FieldValue::Text(s)) => {
                    let line_len = if let CursorState::TextArea { row, .. } = &field.cursor {
                        line_len_at_row(s, *row)
                    } else {
                        0
                    };
                    if let CursorState::TextArea { col, .. } = &mut field.cursor {
                        *col = line_len;
                    }
                }
                _ => {}
            }
        }
    }
}
