use crate::ui::interaction::Mode;
use crate::ui::interaction::{ActionId, Command, InsertMode};
use crate::ui::screens::ScreenState;

use super::{controller::IssueFormController, state::IssueFormState};

#[test]
fn test_move_down_between_fields() {
    let mut state = IssueFormState::new();
    assert_eq!(state.form().selected_index(), 0);

    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveDown,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert_eq!(state.form().selected_index(), 1);
}

#[test]
fn test_move_up_between_fields() {
    let mut state = IssueFormState::new();

    // Move to second field first
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveDown,
            repeat: 1,
        },
        Mode::Normal,
    );
    assert_eq!(state.form().selected_index(), 1);

    // Move back up
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveUp,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert_eq!(state.form().selected_index(), 0);
}

#[test]
fn test_move_top() {
    let mut state = IssueFormState::new();

    // Move down multiple times
    for _ in 0..5 {
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveDown,
                repeat: 1,
            },
            Mode::Normal,
        );
    }
    assert_eq!(state.form().selected_index(), 5);

    // Move to top
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveTop,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert_eq!(state.form().selected_index(), 0);
}

#[test]
fn test_move_bottom() {
    let mut state = IssueFormState::new();
    assert_eq!(state.form().selected_index(), 0);

    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveBottom,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    let total_fields = state.form().fields().len();
    assert_eq!(state.form().selected_index(), total_fields - 1);
}

#[test]
fn test_enter_insert_mode() {
    let mut state = IssueFormState::new();

    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::EnterInsert(InsertMode::Before),
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    // Cursor should be at position 0 for text field
    assert!(matches!(
        state.form().selected_field().unwrap().cursor,
        crate::ui::components::form::CursorState::Text { position: 0 }
    ));
}

#[test]
fn test_text_input() {
    use crossterm::event::KeyCode;
    let mut state = IssueFormState::new();

    // Enter insert mode first
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::EnterInsert(InsertMode::Before),
            repeat: 1,
        },
        Mode::Normal,
    );

    // Type some text
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::RawInput(KeyCode::Char('H')),
            repeat: 1,
        },
        Mode::Insert,
    );
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::RawInput(KeyCode::Char('i')),
            repeat: 1,
        },
        Mode::Insert,
    );

    let field = state.form().selected_field().unwrap();
    assert_eq!(field.value.as_text(), Some("Hi"));
}

#[test]
fn test_cursor_movement() {
    let mut state = IssueFormState::new();

    // Enter insert mode and type text
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::EnterInsert(InsertMode::Before),
            repeat: 1,
        },
        Mode::Normal,
    );

    // Type "Hello"
    for ch in "Hello".chars() {
        use crossterm::event::KeyCode;
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::RawInput(KeyCode::Char(ch)),
                repeat: 1,
            },
            Mode::Insert,
        );
    }

    // Move cursor left
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveLeft,
            repeat: 2,
        },
        Mode::Normal,
    );

    if let crate::ui::components::form::CursorState::Text { position } =
        state.form().selected_field().unwrap().cursor
    {
        assert_eq!(position, 3); // "Hel|lo"
    } else {
        panic!("Expected Text cursor state");
    }
}

#[test]
fn test_dropdown_expand() {
    let mut state = IssueFormState::new();

    // Move to Issue Type field (index 2)
    for _ in 0..2 {
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveDown,
                repeat: 1,
            },
            Mode::Normal,
        );
    }

    // Confirm to expand dropdown
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::Confirm,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert!(
        state
            .form()
            .selected_field()
            .unwrap()
            .field_type
            .is_expanded()
    );
}

#[test]
fn test_dropdown_navigation() {
    let mut state = IssueFormState::new();

    // Move to Issue Type field and expand
    for _ in 0..2 {
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveDown,
                repeat: 1,
            },
            Mode::Normal,
        );
    }
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::Confirm,
            repeat: 1,
        },
        Mode::Normal,
    );

    // Navigate within dropdown
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveDown,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    // Cursor should move within dropdown, not to next field
    assert_eq!(state.form().selected_index(), 2);
}

#[test]
fn test_required_field_validation() {
    let state = IssueFormState::new();

    // Summary field (index 0) is required
    let field = &state.form().fields()[0];
    assert!(field.required);

    // Validate empty required field
    let validation = field.validate();
    assert!(validation.is_some());
}

#[test]
fn test_command_line_quit() {
    use crate::ui::screens::CommandLineCommand;
    let mut state = IssueFormState::new();

    let result = IssueFormController::handle_command_line(&mut state, CommandLineCommand::Quit);

    assert_eq!(result, ScreenState::Close);
}

#[test]
fn test_command_line_write_with_validation_error() {
    use crate::ui::screens::CommandLineCommand;
    let mut state = IssueFormState::new();

    // Don't fill required Summary field
    let result = IssueFormController::handle_command_line(&mut state, CommandLineCommand::Write);

    // Should show validation error, not close
    assert_eq!(result, ScreenState::Refresh);
    assert!(state.error().is_some());
}

#[test]
fn test_command_line_write_quit_with_validation_error() {
    use crate::ui::screens::CommandLineCommand;
    let mut state = IssueFormState::new();

    let result =
        IssueFormController::handle_command_line(&mut state, CommandLineCommand::WriteQuit);

    assert_eq!(result, ScreenState::Refresh);
    assert!(state.error().is_some());
}

#[test]
fn test_command_line_write_quit_should_close_when_summary_present() {
    use crate::ui::components::form::FieldValue;
    use crate::ui::screens::CommandLineCommand;

    let mut state = IssueFormState::new();
    if let FieldValue::Text(summary) = &mut state.form_mut().fields_mut()[0].value {
        *summary = "Valid summary".to_string();
    }

    let result =
        IssueFormController::handle_command_line(&mut state, CommandLineCommand::WriteQuit);

    assert_eq!(result, ScreenState::Close);
    assert!(state.error().is_none());
}

#[test]
fn test_repeat_count() {
    let mut state = IssueFormState::new();
    assert_eq!(state.form().selected_index(), 0);

    // Move down 3 times with one command
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::MoveDown,
            repeat: 3,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert_eq!(state.form().selected_index(), 3);
}

#[test]
fn test_text_popup_open_close() {
    use crossterm::event::KeyCode;
    let mut state = IssueFormState::new();

    // Open text popup by pressing Enter on Summary field
    // This should also switch to Insert mode
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::Confirm,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::SwitchMode(Mode::Insert));
    assert!(state.is_text_popup_open());

    // Press Esc in Insert mode - should NOT close popup (only switches to Normal mode)
    let _result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::RawInput(KeyCode::Esc),
            repeat: 1,
        },
        Mode::Insert,
    );

    // In Insert mode, Esc is handled by dropdown check, returns Stay or processes normally
    assert!(state.is_text_popup_open()); // Still open

    // Press Esc in Normal mode - should close popup
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::RawInput(KeyCode::Esc),
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert!(!state.is_text_popup_open()); // Now closed
}

#[test]
fn test_dropdown_close_with_q() {
    let mut state = IssueFormState::new();

    // Move to Issue Type field (index 2 - Select field)
    for _ in 0..2 {
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveDown,
                repeat: 1,
            },
            Mode::Normal,
        );
    }

    // Open dropdown
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::Confirm,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert!(
        state
            .form()
            .selected_field()
            .unwrap()
            .field_type
            .is_expanded()
    );

    // Press 'q' in Normal mode to close dropdown
    let result = IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::Quit,
            repeat: 1,
        },
        Mode::Normal,
    );

    assert_eq!(result, ScreenState::Refresh);
    assert!(
        !state
            .form()
            .selected_field()
            .unwrap()
            .field_type
            .is_expanded()
    );
}

#[test]
fn test_date_mask() {
    use crossterm::event::KeyCode;
    let mut state = IssueFormState::new();

    // Move to Due Date field (index 13)
    for _ in 0..13 {
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveDown,
                repeat: 1,
            },
            Mode::Normal,
        );
    }

    // Enter insert mode
    IssueFormController::handle_command(
        &mut state,
        Command {
            action: ActionId::Confirm,
            repeat: 1,
        },
        Mode::Normal,
    );

    // Type digits: 2, 0, 2, 4, 0, 3, 1, 5
    let digits = ['2', '0', '2', '4', '0', '3', '1', '5'];
    for digit in digits.iter() {
        IssueFormController::handle_command(
            &mut state,
            Command {
                action: ActionId::RawInput(KeyCode::Char(*digit)),
                repeat: 1,
            },
            Mode::Insert,
        );
    }

    // Should be formatted as 2024-03-15
    let field = state.form().selected_field().unwrap();
    assert_eq!(field.value.as_text(), Some("2024-03-15"));
}
