use super::field::FormField;

mod cursor;
mod editing;
mod selection;
mod text_ops;

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
}
