use crate::{
    app::{
        error::AppErrorState,
        key_handlers::{ActionId, Command, InsertMode},
        state::Mode,
    },
    ui::components::form::{CursorState, FieldType, FieldValue, FormState},
    ui::screens::{CommandLineCommand, ScreenState},
};

use super::state::IssueFormState;

pub struct IssueFormController;

impl IssueFormController {
    pub fn handle_command(state: &mut IssueFormState, command: Command, mode: Mode) -> ScreenState {
        state.clear_error();

        // If text popup is open in Normal mode, close it on any non-movement action
        // This handles Esc (via Quit action), q, Enter, etc.
        if state.is_text_popup_open() && mode == Mode::Normal {
            let is_movement = matches!(
                command.action,
                ActionId::MoveUp
                    | ActionId::MoveDown
                    | ActionId::MoveLeft
                    | ActionId::MoveRight
                    | ActionId::MoveTop
                    | ActionId::MoveBottom
                    | ActionId::MoveLineStart
                    | ActionId::MoveLineEnd
                    | ActionId::MoveWordForward
                    | ActionId::MoveWordBackward
                    | ActionId::MoveWordEnd
            );

            if !is_movement {
                // Close popup and don't process the action that triggered the close
                state.close_text_popup();
                return ScreenState::Refresh;
            }
        }

        match command.action {
            ActionId::MoveDown => {
                // Check if dropdown is expanded
                if let Some(field) = state.form().selected_field() {
                    if field.field_type.is_expanded() {
                        // Navigate dropdown with j/k
                        state.form_mut().move_selection_down();
                        return ScreenState::Refresh;
                    }
                }
                // Otherwise move to next field
                for _ in 0..command.repeat {
                    state.form_mut().move_next();
                }
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                // Check if dropdown is expanded
                if let Some(field) = state.form().selected_field() {
                    if field.field_type.is_expanded() {
                        // Navigate dropdown with j/k
                        state.form_mut().move_selection_up();
                        return ScreenState::Refresh;
                    }
                }
                // Otherwise move to previous field
                for _ in 0..command.repeat {
                    state.form_mut().move_prev();
                }
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.form_mut().move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.form_mut().move_bottom();
                ScreenState::Refresh
            }
            ActionId::MoveLeft => {
                state.form_mut().move_cursor_left(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveRight => {
                state.form_mut().move_cursor_right(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveLineStart => {
                state.form_mut().move_cursor_line_start();
                ScreenState::Refresh
            }
            ActionId::MoveLineEnd => {
                state.form_mut().move_cursor_line_end();
                ScreenState::Refresh
            }
            ActionId::MoveWordForward => {
                state.form_mut().move_word_right(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveWordBackward => {
                state.form_mut().move_word_left(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveWordEnd => {
                state.form_mut().move_word_end(command.repeat);
                ScreenState::Refresh
            }
            ActionId::EnterInsert(mode) => {
                let form = state.form_mut();
                match mode {
                    InsertMode::Before => form.enter_insert_before(),
                    InsertMode::After => form.enter_insert_after(),
                    InsertMode::LineStart => form.enter_insert_line_start(),
                    InsertMode::LineEnd => form.enter_insert_line_end(),
                }
                ScreenState::Refresh
            }
            ActionId::RawInput(c) => {
                handle_raw_input(state.form_mut(), c);
                ScreenState::Refresh
            }
            ActionId::Confirm => {
                // Enter key: toggle dropdown, select option, or open text popup
                if let Some(field) = state.form().selected_field() {
                    match &field.field_type {
                        FieldType::Select { .. } | FieldType::MultiSelect { .. } => {
                            if field.field_type.is_expanded() {
                                // Dropdown is open: select current option
                                state.form_mut().select_option();
                            } else {
                                // Dropdown is closed: open it
                                state.form_mut().toggle_dropdown();
                            }
                            return ScreenState::Refresh;
                        }
                        FieldType::Text { .. } | FieldType::TextArea { .. } => {
                            // Open popup only for Summary and Description fields
                            let should_open_popup =
                                field.label == "Summary" || field.label == "Description";

                            if should_open_popup {
                                state.form_mut().enter_insert_after();
                                state.open_text_popup();
                                return ScreenState::SwitchMode(Mode::Insert);
                            } else {
                                // For other text fields, just enter insert mode inline
                                state.form_mut().enter_insert_after();
                                return ScreenState::SwitchMode(Mode::Insert);
                            }
                        }
                    }
                }
                ScreenState::Stay
            }
            ActionId::Quit => {
                // In Normal mode with dropdown open, 'q' closes dropdown instead of quitting
                if mode == Mode::Normal {
                    if let Some(field) = state.form().selected_field() {
                        if field.field_type.is_expanded() {
                            state.form_mut().toggle_dropdown();
                            return ScreenState::Refresh;
                        }
                    }
                }
                // Otherwise, let the default handler deal with it (stays or closes)
                ScreenState::Stay
            }
            _ => ScreenState::Stay,
        }
    }

    pub fn handle_command_line(state: &mut IssueFormState, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => match validate_form(state) {
                Ok(_) => {
                    // TODO: Save issue to database
                    // For now, just show success and close
                    ScreenState::Close
                }
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::WriteQuit => match validate_form(state) {
                Ok(_) => {
                    // TODO: Save issue to database
                    ScreenState::Close
                }
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

fn validate_form(state: &IssueFormState) -> Result<(), String> {
    // Validate all required fields
    let mut errors = Vec::new();

    for field in state.form().fields() {
        if let Some(error) = field.validate() {
            errors.push(error.message);
        }
    }

    if !errors.is_empty() {
        return Err(errors.join(", "));
    }

    // Summary is required (field index 0)
    let summary = state
        .form()
        .fields()
        .get(0)
        .and_then(|f| f.value.as_text())
        .unwrap_or("");

    if summary.trim().is_empty() {
        return Err("Summary is required".to_string());
    }

    Ok(())
}

fn handle_raw_input(form: &mut FormState, code: crossterm::event::KeyCode) {
    // Check if we're in a dropdown
    if let Some(field) = form.selected_field() {
        if field.field_type.is_expanded() {
            match code {
                // Esc closes dropdown
                crossterm::event::KeyCode::Esc => {
                    form.toggle_dropdown();
                    return;
                }
                // Space in MultiSelect toggles checkbox
                crossterm::event::KeyCode::Char(' ') => {
                    if matches!(field.field_type, FieldType::MultiSelect { .. }) {
                        form.select_option();
                        return;
                    }
                }
                _ => {}
            }
            // Other keys are ignored when dropdown is open
            return;
        }
    }

    // Check if we're editing the Due Date field
    let is_due_date = form
        .selected_field()
        .map(|f| f.label == "Due Date (YYYY-MM-DD)")
        .unwrap_or(false);

    // Normal text input
    match code {
        crossterm::event::KeyCode::Char(ch) => {
            if is_due_date && ch.is_ascii_digit() {
                // Apply date mask for Due Date field
                handle_date_input(form, ch);
            } else if is_due_date && ch == '-' {
                // Allow manual dash input
                form.insert_char(ch);
            } else if !is_due_date {
                // For non-date fields, insert any character
                form.insert_char(ch);
            }
            // Ignore non-digit characters in date field
        }
        crossterm::event::KeyCode::Tab => form.insert_char('\t'),
        crossterm::event::KeyCode::Enter => form.insert_char('\n'),
        crossterm::event::KeyCode::Backspace => form.backspace(),
        crossterm::event::KeyCode::Delete => form.delete(),
        _ => {}
    }
}

fn handle_date_input(form: &mut FormState, digit: char) {
    // Get current value
    let current = form
        .selected_field()
        .and_then(|f| f.value.as_text())
        .unwrap_or("")
        .to_string();

    // Remove all dashes to get just digits
    let digits_only: String = current.chars().filter(|c| c.is_ascii_digit()).collect();

    // Add the new digit
    let mut new_digits = digits_only;
    new_digits.push(digit);

    // Don't allow more than 8 digits (YYYYMMDD)
    if new_digits.len() > 8 {
        return;
    }

    // Format with dashes: YYYY-MM-DD
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

    // Update field value and cursor position
    if let Some(field) = form.selected_field_mut() {
        if let FieldValue::Text(value) = &mut field.value {
            *value = formatted.clone();
            // Set cursor to end
            if let CursorState::Text { position } = &mut field.cursor {
                *position = formatted.len();
            }
        }
    }
}
