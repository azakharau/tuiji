use super::field::FormField;
use super::field_type::{CursorState, FieldType, FieldValue};

pub struct FormState {
    fields: Vec<FormField>,
    selected_index: usize,
}

impl FormState {
    pub fn new(fields: Vec<FormField>) -> Self {
        Self {
            fields,
            selected_index: 0,
        }
    }

    pub fn fields(&self) -> &[FormField] {
        &self.fields
    }

    pub fn fields_mut(&mut self) -> &mut [FormField] {
        &mut self.fields
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_field(&self) -> Option<&FormField> {
        self.fields.get(self.selected_index)
    }

    pub fn selected_field_mut(&mut self) -> Option<&mut FormField> {
        self.fields.get_mut(self.selected_index)
    }

    pub fn move_next(&mut self) {
        if self.selected_index + 1 < self.fields.len() {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn move_prev(&mut self) {
        if self.fields.is_empty() {
            self.selected_index = 0;
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.fields.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn move_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_bottom(&mut self) {
        if !self.fields.is_empty() {
            self.selected_index = self.fields.len() - 1;
        }
    }

    // Text field navigation
    pub fn move_cursor_left(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.field_type, &mut field.cursor, &field.value) {
                (FieldType::Text { .. }, CursorState::Text { position }, _) => {
                    *position = position.saturating_sub(repeat);
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
                            *col -= 1;
                        } else if *row > 0 {
                            // Move to end of previous line
                            *row -= 1;
                            *col = lines.get(*row).map(|l| l.len()).unwrap_or(0);
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
                    let max = s.len();
                    *position = (*position + repeat).min(max);
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
                            *col += 1;
                        } else if *row + 1 < lines.len() {
                            // Move to start of next line
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
            // Clamp column to line length
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
            // Clamp column to line length
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
                    *col = lines.get(*row).map(|l| l.len()).unwrap_or(0);
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

    pub fn insert_char(&mut self, ch: char) {
        if let Some(field) = self.selected_field_mut() {
            match (&mut field.cursor, &mut field.value) {
                (CursorState::Text { position }, FieldValue::Text(s)) => {
                    s.insert(*position, ch);
                    *position += 1;
                }
                (CursorState::TextArea { row, col }, FieldValue::Text(s)) => {
                    // Convert (row, col) to absolute position
                    let pos = row_col_to_position(s, *row, *col);
                    s.insert(pos, ch);

                    if ch == '\n' {
                        // Move to start of next line
                        *row += 1;
                        *col = 0;
                    } else {
                        // Move cursor forward
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
                        // Delete character on current line
                        let pos = row_col_to_position(s, *row, *col);
                        s.remove(pos - 1);
                        *col -= 1;
                    } else if *row > 0 {
                        // At start of line, merge with previous line
                        let lines: Vec<&str> = s.lines().collect();
                        let prev_line_len = lines.get(*row - 1).map(|l| l.len()).unwrap_or(0);

                        let pos = row_col_to_position(s, *row, *col);
                        if pos > 0 {
                            s.remove(pos - 1); // Remove newline
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
                    if let CursorState::TextArea { col, .. } = &mut field.cursor {
                        *col = s.len();
                    }
                }
                _ => {}
            }
        }
    }

    // Select/MultiSelect navigation
    pub fn toggle_dropdown(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            let expanded = field.field_type.is_expanded();
            field.field_type.set_expanded(!expanded);
        }
    }

    pub fn select_option(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.field_type, &mut field.cursor, &mut field.value) {
                (
                    FieldType::Select { options, .. },
                    CursorState::Select { index },
                    FieldValue::Single(selected),
                ) => {
                    if let Some(option) = options.get(*index) {
                        *selected = Some(option.value.clone());
                        field.field_type.set_expanded(false);
                    }
                }
                (
                    FieldType::MultiSelect { options: _, .. },
                    CursorState::MultiSelect { index },
                    FieldValue::Multiple(selected),
                ) => {
                    if let Some(options_mut) = field.field_type.options_mut()
                        && let Some(option) = options_mut.get_mut(*index)
                    {
                        option.selected = !option.selected;
                        if option.selected {
                            if !selected.contains(&option.value) {
                                selected.push(option.value.clone());
                            }
                        } else {
                            selected.retain(|v| v != &option.value);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn move_selection_up(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match &mut field.cursor {
                CursorState::Select { index } => {
                    *index = index.saturating_sub(1);
                }
                CursorState::MultiSelect { index } => {
                    *index = index.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            match (&field.field_type, &mut field.cursor) {
                (FieldType::Select { options, .. }, CursorState::Select { index }) => {
                    if *index + 1 < options.len() {
                        *index += 1;
                    }
                }
                (FieldType::MultiSelect { options, .. }, CursorState::MultiSelect { index }) => {
                    if *index + 1 < options.len() {
                        *index += 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn word_forward(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    let len = value.len();
    if pos >= len {
        return len;
    }

    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len());

    if idx >= chars.len() {
        return len;
    }

    if is_word_char(chars[idx].1) {
        while idx < chars.len() && is_word_char(chars[idx].1) {
            idx += 1;
        }
    }

    while idx < chars.len() && !is_word_char(chars[idx].1) {
        idx += 1;
    }

    if idx < chars.len() { chars[idx].0 } else { len }
}

fn word_end(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    let len = value.len();
    if pos >= len {
        return len;
    }
    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len().saturating_sub(1));

    if idx >= chars.len() {
        return len;
    }

    if !is_word_char(chars[idx].1) {
        while idx < chars.len() && !is_word_char(chars[idx].1) {
            idx += 1;
        }
    }
    if idx >= chars.len() {
        return len;
    }
    while idx + 1 < chars.len() && is_word_char(chars[idx + 1].1) {
        idx += 1;
    }
    chars[idx].0
}

fn word_backward(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    if pos == 0 {
        return 0;
    }
    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len());
    idx = idx.saturating_sub(1);

    if !is_word_char(chars[idx].1) {
        while idx > 0 && !is_word_char(chars[idx].1) {
            idx -= 1;
        }
        if !is_word_char(chars[idx].1) {
            return 0;
        }
    }

    while idx > 0 && is_word_char(chars[idx - 1].1) {
        idx -= 1;
    }
    chars[idx].0
}

// Helper functions for word navigation

/// Convert (row, col) to absolute position in text
fn row_col_to_position(text: &str, row: usize, col: usize) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let mut pos = 0;

    for (i, line) in lines.iter().enumerate() {
        if i == row {
            return pos + col.min(line.len());
        }
        pos += line.len() + 1; // +1 for newline
    }

    // If row is beyond available lines, return end of text
    text.len()
}
