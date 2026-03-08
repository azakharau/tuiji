use std::sync::Arc;

use color_eyre::Result;

use crate::{
    app::{
        AppState, ProfileEditorIntent,
        key_handlers::{Command, KeyBindings, KeyHandler},
        screen_manager::ScreenManager,
        state::ScreenType,
    },
    config::AppConfigState,
    data::RepositoryHub,
    ui::{interaction::ActionId, screens::ScreenState},
};

use super::configuration;

pub fn handle_profiles_command(
    cmd: Command,
    state: &mut AppState,
    cfg_state: &mut AppConfigState,
    key_bindings: &mut Arc<KeyBindings>,
    repo: &mut Option<Arc<RepositoryHub>>,
    screen_manager: &mut ScreenManager,
) -> Result<ScreenState> {
    match cmd.action {
        ActionId::MoveUp => {
            let screen = screen_manager.ensure_profiles(cfg_state)?;
            screen.move_up(cmd.repeat);
            return Ok(ScreenState::Refresh);
        }
        ActionId::MoveDown => {
            let screen = screen_manager.ensure_profiles(cfg_state)?;
            screen.move_down(cmd.repeat);
            return Ok(ScreenState::Refresh);
        }
        ActionId::MoveTop => {
            let screen = screen_manager.ensure_profiles(cfg_state)?;
            screen.move_top();
            return Ok(ScreenState::Refresh);
        }
        ActionId::MoveBottom => {
            let screen = screen_manager.ensure_profiles(cfg_state)?;
            screen.move_bottom();
            return Ok(ScreenState::Refresh);
        }
        _ => {}
    }

    let (is_empty, selected_menu, selected_profile) = {
        let screen = screen_manager.ensure_profiles(cfg_state)?;
        (
            screen.is_empty(),
            screen.selected_menu_id().map(str::to_owned),
            screen.selected_profile_id().map(str::to_owned),
        )
    };

    match cmd.action {
        ActionId::Confirm => {
            if is_empty {
                if matches!(selected_menu.as_deref(), Some("quit")) {
                    return Ok(ScreenState::Quit);
                }
                start_profile_creation(state, screen_manager, ProfileEditorIntent::New);
                return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
            }
            let Some(profile_id) = selected_profile else {
                return Ok(ScreenState::Stay);
            };
            let mut cfg = configuration::cfg_or_default(cfg_state);
            if cfg.profiles.iter().any(|p| p.id == profile_id) {
                cfg.set_active_profile(profile_id.as_str());
                configuration::save_config(cfg, cfg_state, key_bindings, repo, screen_manager)?;
            }
            Ok(ScreenState::Refresh)
        }
        ActionId::EditProfile => {
            if is_empty {
                start_profile_creation(state, screen_manager, ProfileEditorIntent::New);
                return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
            }
            let can_edit_selected = if let (AppConfigState::Loaded(cfg), Some(id)) =
                (&*cfg_state, selected_profile.as_deref())
            {
                cfg.profiles.iter().any(|p| p.id == id)
            } else {
                false
            };
            if can_edit_selected && let Some(id) = selected_profile {
                start_profile_creation(state, screen_manager, ProfileEditorIntent::Edit(id));
                return Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation));
            }
            Ok(ScreenState::Stay)
        }
        ActionId::DeleteProfile => {
            if is_empty {
                return Ok(ScreenState::Stay);
            }
            let Some(profile_id) = selected_profile else {
                return Ok(ScreenState::Stay);
            };
            let mut cfg = configuration::cfg_or_default(cfg_state);
            if cfg.remove_profile(profile_id.as_str()) {
                configuration::save_config(cfg, cfg_state, key_bindings, repo, screen_manager)?;
                return Ok(ScreenState::Refresh);
            }
            Ok(ScreenState::Stay)
        }
        ActionId::NewProfile => {
            start_profile_creation(state, screen_manager, ProfileEditorIntent::New);
            Ok(ScreenState::SwitchTo(ScreenType::ProfileCreation))
        }
        _ => Ok(ScreenState::Stay),
    }
}

pub fn handle_settings_command(
    cmd: Command,
    cfg_state: &AppConfigState,
    screen_manager: &mut ScreenManager,
) -> Result<ScreenState> {
    let screen = screen_manager.ensure_settings(cfg_state)?;
    Ok(screen.handle_command(cmd))
}

pub fn start_profile_creation(
    state: &mut AppState,
    screen_manager: &mut ScreenManager,
    intent: ProfileEditorIntent,
) {
    state.profile_editor = Some(intent);
    screen_manager.invalidate(ScreenType::ProfileCreation);
}
