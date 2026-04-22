use crate::{
    ui::components::form::FieldType,
    ui::screens::ScreenState,
    ui::{
        interaction::Mode,
        interaction::{ActionId, Command, InsertMode},
    },
};

use super::{IssueFormController, raw_input::handle_raw_input};
use crate::ui::screens::issue_form::state::{IssueFormState, IssueFormSurface};

impl IssueFormController {
    pub fn handle_command(state: &mut IssueFormState, command: Command, mode: Mode) -> ScreenState {
        state.clear_error();

        if mode == Mode::Normal
            && matches!(
                command.action,
                ActionId::RawInput(crossterm::event::KeyCode::Esc)
            )
            && !matches!(state.active_surface(), IssueFormSurface::Form)
        {
            state.close_active_overlay();
            return ScreenState::Refresh;
        }

        match state.active_surface() {
            IssueFormSurface::Form => handle_form_surface(state, command, mode),
            IssueFormSurface::TextPopup { .. } => handle_text_popup_surface(state, command, mode),
            IssueFormSurface::Dropdown { .. } => handle_dropdown_surface(state, command, mode),
        }
    }
}

fn handle_confirm(state: &mut IssueFormState) -> ScreenState {
    if let Some(field) = state.form().selected_field() {
        match &field.field_type {
            FieldType::Select { .. } | FieldType::MultiSelect { .. } => {
                state.open_dropdown();
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

fn handle_form_surface(state: &mut IssueFormState, command: Command, _mode: Mode) -> ScreenState {
    match command.action {
        ActionId::MoveDown => {
            for _ in 0..command.repeat {
                state.form_mut().move_next();
            }
            ScreenState::Refresh
        }
        ActionId::MoveUp => {
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
            enter_insert_mode(state, insert_mode);
            ScreenState::Refresh
        }
        ActionId::RawInput(code) => {
            handle_raw_input(state, code);
            ScreenState::Refresh
        }
        ActionId::Confirm => handle_confirm(state),
        ActionId::Quit => ScreenState::Stay,
        _ => ScreenState::Stay,
    }
}

fn handle_text_popup_surface(
    state: &mut IssueFormState,
    command: Command,
    mode: Mode,
) -> ScreenState {
    match command.action {
        ActionId::MoveUp => {
            state.form_mut().move_cursor_up(command.repeat);
            ScreenState::Refresh
        }
        ActionId::MoveDown => {
            state.form_mut().move_cursor_down(command.repeat);
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
        ActionId::MoveTop | ActionId::MoveLineStart => {
            state.form_mut().move_cursor_line_start();
            ScreenState::Refresh
        }
        ActionId::MoveBottom | ActionId::MoveLineEnd => {
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
            enter_insert_mode(state, insert_mode);
            ScreenState::Refresh
        }
        ActionId::RawInput(code) => {
            handle_raw_input(state, code);
            ScreenState::Refresh
        }
        ActionId::Quit if mode == Mode::Normal => {
            state.close_text_popup();
            ScreenState::Refresh
        }
        ActionId::Confirm => ScreenState::SwitchMode(Mode::Insert),
        _ => ScreenState::Stay,
    }
}

fn handle_dropdown_surface(
    state: &mut IssueFormState,
    command: Command,
    mode: Mode,
) -> ScreenState {
    match command.action {
        ActionId::MoveUp => {
            for _ in 0..command.repeat {
                state.form_mut().move_selection_up();
            }
            ScreenState::Refresh
        }
        ActionId::MoveDown => {
            for _ in 0..command.repeat {
                state.form_mut().move_selection_down();
            }
            ScreenState::Refresh
        }
        ActionId::MoveTop => {
            state.form_mut().move_selection_top();
            ScreenState::Refresh
        }
        ActionId::MoveBottom => {
            state.form_mut().move_selection_bottom();
            ScreenState::Refresh
        }
        ActionId::Confirm => {
            state.form_mut().select_option();
            if let Some(field) = state.form().selected_field()
                && matches!(field.field_type, FieldType::Select { .. })
            {
                state.close_dropdown();
            }
            ScreenState::Refresh
        }
        ActionId::RawInput(code) => {
            handle_raw_input(state, code);
            ScreenState::Refresh
        }
        ActionId::Quit if mode == Mode::Normal => {
            state.close_dropdown();
            ScreenState::Refresh
        }
        _ => ScreenState::Stay,
    }
}

fn enter_insert_mode(state: &mut IssueFormState, insert_mode: InsertMode) {
    let form = state.form_mut();
    match insert_mode {
        InsertMode::Before => form.enter_insert_before(),
        InsertMode::After => form.enter_insert_after(),
        InsertMode::LineStart => form.enter_insert_line_start(),
        InsertMode::LineEnd => form.enter_insert_line_end(),
    }
}
