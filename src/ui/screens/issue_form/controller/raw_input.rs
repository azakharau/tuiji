use crossterm::event::KeyCode;

use crate::ui::components::form::{FieldType, FormState};
use crate::ui::screens::issue_form::state::IssueFormState;

pub(super) fn handle_raw_input(state: &mut IssueFormState, code: KeyCode) {
    if state.is_dropdown_open() {
        handle_dropdown_raw_input(state.form_mut(), code);
        return;
    }

    handle_text_raw_input(state.form_mut(), code);
}

fn handle_dropdown_raw_input(form: &mut FormState, code: KeyCode) {
    if matches!(code, KeyCode::Char(' '))
        && let Some(field) = form.selected_field()
        && matches!(field.field_type, FieldType::MultiSelect { .. })
    {
        form.select_option();
    }
}

fn handle_text_raw_input(form: &mut FormState, code: KeyCode) {
    match code {
        KeyCode::Char(ch) => form.insert_char(ch),
        KeyCode::Tab => form.insert_char('\t'),
        KeyCode::Enter => form.insert_char('\n'),
        KeyCode::Backspace => form.backspace(),
        KeyCode::Delete => form.delete(),
        _ => {}
    }
}
