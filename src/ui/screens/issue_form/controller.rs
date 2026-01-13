use crate::{
    app::{
        error::AppErrorState,
        key_handlers::{ActionId, Command, InsertMode},
        state::Mode,
    },
    data::IssueSummary,
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
                if let Some(field) = state.form().selected_field()
                    && field.field_type.is_expanded()
                {
                    // Navigate dropdown with j/k
                    state.form_mut().move_selection_down();
                    return ScreenState::Refresh;
                }
                // Otherwise move to next field
                for _ in 0..command.repeat {
                    state.form_mut().move_next();
                }
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                // Check if dropdown is expanded
                if let Some(field) = state.form().selected_field()
                    && field.field_type.is_expanded()
                {
                    // Navigate dropdown with j/k
                    state.form_mut().move_selection_up();
                    return ScreenState::Refresh;
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
                if mode == Mode::Normal
                    && let Some(field) = state.form().selected_field()
                    && field.field_type.is_expanded()
                {
                    state.form_mut().toggle_dropdown();
                    return ScreenState::Refresh;
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
                    // Create issue from form data
                    let issue = form_to_issue(state);
                    ScreenState::CreateIssue(Box::new(issue))
                }
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::WriteQuit => match validate_form(state) {
                Ok(_) => {
                    // Create issue from form data
                    let issue = form_to_issue(state);
                    ScreenState::CreateIssue(Box::new(issue))
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
        .first()
        .and_then(|f| f.value.as_text())
        .unwrap_or("");

    if summary.trim().is_empty() {
        return Err("Summary is required".to_string());
    }

    Ok(())
}

fn handle_raw_input(form: &mut FormState, code: crossterm::event::KeyCode) {
    // Check if we're in a dropdown
    if let Some(field) = form.selected_field()
        && field.field_type.is_expanded()
    {
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
    if let Some(field) = form.selected_field_mut()
        && let FieldValue::Text(value) = &mut field.value
    {
        *value = formatted.clone();
        // Set cursor to end
        if let CursorState::Text { position } = &mut field.cursor {
            *position = formatted.len();
        }
    }
}

/// Convert form data to IssueSummary for saving
fn form_to_issue(state: &IssueFormState) -> IssueSummary {
    let form = state.form();
    let fields = form.fields();

    // Generate temporary key for offline-created issues
    // Format: TEMP-{timestamp}-{random}
    let temp_key = format!(
        "TEMP-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    // Extract values from form fields (indices match state.rs field order)
    let summary = fields[0].value.as_text().unwrap_or("").to_string();
    let description = fields[1].value.as_text().map(|s| s.to_string());
    let issue_type = fields[2].value.as_single().unwrap_or("story").to_string();
    let status = fields[3].value.as_single().unwrap_or("todo").to_string();
    let priority = fields[4].value.as_single().unwrap_or("medium").to_string();
    let assignee_raw = fields[5].value.as_single().unwrap_or("unassigned");
    let assignee = if assignee_raw == "unassigned" {
        "Unassigned".to_string()
    } else {
        assignee_raw.to_string()
    };
    let reporter = fields[6].value.as_single().map(|s| s.to_string());
    let labels = fields[7].value.as_multiple().unwrap_or(&[]).to_vec();
    // Components are at index 8 but we don't have a field for them in IssueSummary
    let story_points_str = fields[9].value.as_text().unwrap_or("");
    let story_points = story_points_str.parse::<f64>().ok();
    let sprint_str = fields[10].value.as_single().unwrap_or("none");
    let sprint_id = if sprint_str != "none" {
        // Extract sprint number from "sprint-23" format
        sprint_str
            .strip_prefix("sprint-")
            .and_then(|s| s.parse::<i64>().ok())
    } else {
        None
    };
    let epic = fields[11].value.as_single().and_then(|e| {
        if e != "none" {
            Some(e.to_string())
        } else {
            None
        }
    });
    let environment = fields[12].value.as_text().map(|s| s.to_string());
    let _due_date_str = fields[13].value.as_text().unwrap_or("");
    // TODO: Parse due date from YYYY-MM-DD format

    let now = std::time::SystemTime::now();

    IssueSummary {
        key: temp_key,
        summary,
        epic,
        status,
        issue_type,
        assignee,
        priority,
        story_points,
        project_key: None, // Will be set based on selected board
        sprint_id,
        updated_at: Some(now),
        comments: Vec::new(),
        dirty: true, // Mark as dirty since it's locally created
        conflict: false,
        remote_snapshot: None,
        description,
        reporter,
        creator: None,
        created_at: Some(now),
        resolution_date: None,
        resolution: None,
        labels,
        fix_versions: Vec::new(),
        parent_key: None,
        environment,
        time_estimate: None,
        time_spent: None,
        time_remaining: None,
        custom_fields: None,
    }
}
