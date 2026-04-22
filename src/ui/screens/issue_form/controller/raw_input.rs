use crossterm::event::KeyCode;

use crate::ui::components::form::{CursorState, FieldType, FieldValue, FormState};
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
    let is_due_date = form
        .selected_field()
        .map(|field| field.label == "Due Date (YYYY-MM-DD)")
        .unwrap_or(false);

    match code {
        KeyCode::Char(ch) => {
            if is_due_date {
                if ch.is_ascii_digit() {
                    handle_date_input(form, ch);
                } else if ch == '-' {
                    form.insert_char(ch);
                }
            } else {
                form.insert_char(ch);
            }
        }
        KeyCode::Tab => form.insert_char('\t'),
        KeyCode::Enter => form.insert_char('\n'),
        KeyCode::Backspace => form.backspace(),
        KeyCode::Delete => form.delete(),
        _ => {}
    }
}

fn handle_date_input(form: &mut FormState, digit: char) {
    let current = form
        .selected_field()
        .and_then(|field| field.value.as_text())
        .unwrap_or("")
        .to_string();

    let digits_only: String = current.chars().filter(|ch| ch.is_ascii_digit()).collect();

    let mut new_digits = digits_only;
    new_digits.push(digit);

    if new_digits.len() > 8 {
        return;
    }

    let formatted = match new_digits.len() {
        0 => String::new(),
        1..=4 => new_digits,
        5..=6 => format!("{}-{}", &new_digits[0..4], &new_digits[4..]),
        7..=8 => format!(
            "{}-{}-{}",
            &new_digits[0..4],
            &new_digits[4..6],
            &new_digits[6..]
        ),
        _ => new_digits,
    };

    if let Some(field) = form.selected_field_mut()
        && let FieldValue::Text(value) = &mut field.value
    {
        *value = formatted.clone();
        if let CursorState::Text { position } = &mut field.cursor {
            *position = formatted.len();
        }
    }
}
