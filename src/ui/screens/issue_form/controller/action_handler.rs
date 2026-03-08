use crate::{
    ui::components::form::FieldType,
    ui::screens::ScreenState,
    ui::{
        interaction::Mode,
        interaction::{ActionId, Command, InsertMode},
    },
};

use super::{IssueFormController, raw_input::handle_raw_input};
use crate::ui::screens::issue_form::state::IssueFormState;

impl IssueFormController {
    pub fn handle_command(state: &mut IssueFormState, command: Command, mode: Mode) -> ScreenState {
        state.clear_error();

        if state.is_text_popup_open() && mode == Mode::Normal && !is_popup_movement(command.action)
        {
            state.close_text_popup();
            return ScreenState::Refresh;
        }

        match command.action {
            ActionId::MoveDown => {
                if let Some(field) = state.form().selected_field()
                    && field.field_type.is_expanded()
                {
                    state.form_mut().move_selection_down();
                    return ScreenState::Refresh;
                }
                for _ in 0..command.repeat {
                    state.form_mut().move_next();
                }
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                if let Some(field) = state.form().selected_field()
                    && field.field_type.is_expanded()
                {
                    state.form_mut().move_selection_up();
                    return ScreenState::Refresh;
                }
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
            ActionId::EnterInsert(insert_mode) => {
                let form = state.form_mut();
                match insert_mode {
                    InsertMode::Before => form.enter_insert_before(),
                    InsertMode::After => form.enter_insert_after(),
                    InsertMode::LineStart => form.enter_insert_line_start(),
                    InsertMode::LineEnd => form.enter_insert_line_end(),
                }
                ScreenState::Refresh
            }
            ActionId::RawInput(code) => {
                handle_raw_input(state.form_mut(), code);
                ScreenState::Refresh
            }
            ActionId::Confirm => handle_confirm(state),
            ActionId::Quit => {
                if mode == Mode::Normal
                    && let Some(field) = state.form().selected_field()
                    && field.field_type.is_expanded()
                {
                    state.form_mut().toggle_dropdown();
                    return ScreenState::Refresh;
                }
                ScreenState::Stay
            }
            _ => ScreenState::Stay,
        }
    }
}

fn handle_confirm(state: &mut IssueFormState) -> ScreenState {
    if let Some(field) = state.form().selected_field() {
        match &field.field_type {
            FieldType::Select { .. } | FieldType::MultiSelect { .. } => {
                if field.field_type.is_expanded() {
                    state.form_mut().select_option();
                } else {
                    state.form_mut().toggle_dropdown();
                }
                return ScreenState::Refresh;
            }
            FieldType::Text { .. } | FieldType::TextArea { .. } => {
                let should_open_popup = field.label == "Summary" || field.label == "Description";
                state.form_mut().enter_insert_after();
                if should_open_popup {
                    state.open_text_popup();
                }
                return ScreenState::SwitchMode(Mode::Insert);
            }
        }
    }
    ScreenState::Stay
}

fn is_popup_movement(action: ActionId) -> bool {
    matches!(
        action,
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
    )
}
