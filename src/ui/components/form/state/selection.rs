use super::super::field_type::{CursorState, FieldType, FieldValue};
use super::*;

impl FormState {
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
                            selected.retain(|value| value != &option.value);
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
