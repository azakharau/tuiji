use crate::{
    app::{
        error::AppErrorState,
        key_handlers::{ActionId, Command, InsertMode},
    },
    config::{JiraConfig, ProfileConfig},
    ui::components::form::FormState,
    ui::screens::{CommandLineCommand, ScreenState},
};

use super::state::ProfileCreationState;

pub struct ProfileCreationController;

impl ProfileCreationController {
    pub fn handle_command(state: &mut ProfileCreationState, command: Command) -> ScreenState {
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
        state: &mut ProfileCreationState,
        cmd: CommandLineCommand,
    ) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => match build_profile(state) {
                Ok(profile) => ScreenState::SaveProfile(profile),
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::WriteQuit => match build_profile(state) {
                Ok(profile) => ScreenState::SaveProfileAndClose(profile),
                Err(err) => {
                    state.set_error(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

fn build_profile(state: &ProfileCreationState) -> Result<ProfileConfig, String> {
    let name = field_value(state, 0).trim();
    if name.is_empty() {
        return Err("Profile name is required".to_string());
    }

    let jira_url = field_value(state, 1).trim();
    validate_url(jira_url)?;

    let username = field_value(state, 2).trim();
    if username.is_empty() {
        return Err("Email is required".to_string());
    }
    validate_email(username)?;

    let api_token = field_value(state, 3).trim();
    if api_token.is_empty() {
        return Err("Jira API token is required".to_string());
    }

    let id = state
        .profile_id()
        .map(|id| id.to_string())
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    Ok(ProfileConfig {
        id,
        name: name.to_string(),
        jira: JiraConfig {
            base_url: jira_url.to_string(),
            username: username.to_string(),
            api_token: api_token.to_string(),
        },
        sync_mode: state.sync_mode().cloned(),
    })
}

fn field_value(state: &ProfileCreationState, idx: usize) -> &str {
    state
        .form()
        .fields()
        .get(idx)
        .and_then(|item| item.value.as_text())
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

fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("Jira base URL is required".to_string());
    }
    let parsed = reqwest::Url::parse(url).map_err(|_| "Jira base URL is invalid".to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err("Jira base URL must start with http:// or https://".to_string()),
    }
}

fn validate_email(email: &str) -> Result<(), String> {
    if email.chars().any(|c| c.is_whitespace()) {
        return Err("Email must not contain spaces".to_string());
    }
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if local.is_empty() || domain.is_empty() || parts.next().is_some() {
        return Err("Email address is invalid".to_string());
    }
    if domain.starts_with('.') || domain.ends_with('.') || !domain.contains('.') {
        return Err("Email address is invalid".to_string());
    }
    Ok(())
}
