use crate::{
    app::{
        error::AppErrorState,
        key_handlers::{ActionId, Command, InsertMode},
    },
    config::{CustomThemeConfig, ThemePaletteConfig},
    ui::components::form::FormState,
    ui::screens::{CommandLineCommand, ScreenState},
};

use super::state::SettingsThemeFormState;

pub struct SettingsThemeFormController;

impl SettingsThemeFormController {
    pub fn handle_command(state: &mut SettingsThemeFormState, command: Command) -> ScreenState {
        state.clear_error();
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
            _ => ScreenState::Stay,
        }
    }

    pub fn handle_command_line(
        state: &mut SettingsThemeFormState,
        cmd: CommandLineCommand,
    ) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => match build_theme(state) {
                Ok(theme) => ScreenState::SaveCustomTheme(theme),
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::WriteQuit => match build_theme(state) {
                Ok(theme) => ScreenState::SaveCustomThemeAndClose(theme),
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

fn build_theme(state: &SettingsThemeFormState) -> Result<CustomThemeConfig, String> {
    let name = field_value(state, 0).trim();
    if name.is_empty() {
        return Err("Theme name is required".to_string());
    }
    let id = state.unique_theme_id(name);

    Ok(CustomThemeConfig {
        id,
        name: name.to_string(),
        palette: ThemePaletteConfig {
            background: parse_hex(field_value(state, 1), "Background")?,
            text: parse_hex(field_value(state, 2), "Text")?,
            accent: parse_hex(field_value(state, 3), "Accent")?,
            selection: parse_hex(field_value(state, 4), "Selection")?,
            border: parse_hex(field_value(state, 5), "Border")?,
            error: parse_hex(field_value(state, 6), "Error")?,
            warning: parse_hex(field_value(state, 7), "Warning")?,
            info: parse_hex(field_value(state, 8), "Info")?,
            success: parse_hex(field_value(state, 9), "Success")?,
        },
    })
}

fn field_value(state: &SettingsThemeFormState, idx: usize) -> &str {
    state
        .form()
        .fields()
        .get(idx)
        .map(|item| item.value.as_str())
        .unwrap_or("")
}

fn handle_raw_input(form: &mut FormState, code: crossterm::event::KeyCode) {
    match code {
        crossterm::event::KeyCode::Char(ch) => form.insert_char(ch),
        crossterm::event::KeyCode::Tab => form.insert_char('\t'),
        crossterm::event::KeyCode::Backspace => form.backspace(),
        crossterm::event::KeyCode::Delete => form.delete(),
        _ => {}
    }
}

fn parse_hex(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return Err(format!("{label} must be a hex color like #RRGGBB"));
    }
    if u8::from_str_radix(&trimmed[0..2], 16).is_err()
        || u8::from_str_radix(&trimmed[2..4], 16).is_err()
        || u8::from_str_radix(&trimmed[4..6], 16).is_err()
    {
        return Err(format!("{label} must be a hex color like #RRGGBB"));
    }
    Ok(format!("#{}", trimmed.to_lowercase()))
}
