use super::field::FormField;

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

    pub fn move_cursor_left(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            field.cursor_position = field.cursor_position.saturating_sub(repeat);
        }
    }

    pub fn move_cursor_right(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            let max = field.value.len();
            field.cursor_position = (field.cursor_position + repeat).min(max);
        }
    }

    pub fn move_cursor_line_start(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            field.cursor_position = 0;
        }
    }

    pub fn move_cursor_line_end(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            field.cursor_position = field.value.len();
        }
    }

    pub fn move_word_right(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            for _ in 0..repeat {
                field.cursor_position = word_forward(&field.value, field.cursor_position);
            }
        }
    }

    pub fn move_word_end(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            for _ in 0..repeat {
                field.cursor_position = word_end(&field.value, field.cursor_position);
            }
        }
    }

    pub fn move_word_left(&mut self, repeat: usize) {
        if let Some(field) = self.selected_field_mut() {
            for _ in 0..repeat {
                field.cursor_position = word_backward(&field.value, field.cursor_position);
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(field) = self.selected_field_mut() {
            field.value.insert(field.cursor_position, ch);
            field.cursor_position += 1;
        }
    }

    pub fn backspace(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            if field.cursor_position > 0 {
                field.cursor_position -= 1;
                field.value.remove(field.cursor_position);
            }
        }
    }

    pub fn delete(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            if field.cursor_position < field.value.len() {
                field.value.remove(field.cursor_position);
            }
        }
    }

    pub fn enter_insert_before(&mut self) {
        self.move_cursor_line_start();
    }

    pub fn enter_insert_after(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            if field.cursor_position < field.value.len() {
                field.cursor_position += 1;
            }
        }
    }

    pub fn enter_insert_line_start(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            field.cursor_position = 0;
        }
    }

    pub fn enter_insert_line_end(&mut self) {
        if let Some(field) = self.selected_field_mut() {
            field.cursor_position = field.value.len();
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
