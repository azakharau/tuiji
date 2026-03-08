use std::sync::Arc;

use color_eyre::Result;

use crate::{
    app::{
        AppState, error::AppErrorState, key_handlers::KeyBindings,
        notification_service::NotificationService, screen_manager::ScreenManager,
        state::ScreenType,
    },
    config::AppConfigState,
    data::RepositoryHub,
    ui::screens::ScreenState,
};

use super::configuration;

pub struct NormalizeStateDeps<'a> {
    pub state: &'a mut AppState,
    pub cfg_state: &'a mut AppConfigState,
    pub key_bindings: &'a mut Arc<KeyBindings>,
    pub repo: &'a mut Option<Arc<RepositoryHub>>,
    pub screen_manager: &'a mut ScreenManager,
    pub notification_service: &'a mut NotificationService,
}

pub fn normalize_screen_state(
    state: ScreenState,
    close_on_save: bool,
    deps: NormalizeStateDeps<'_>,
) -> Result<ScreenState> {
    let NormalizeStateDeps {
        state: app_state,
        cfg_state,
        key_bindings,
        repo,
        screen_manager,
        notification_service,
    } = deps;

    match state {
        ScreenState::SaveProfile(profile) => {
            if let Err(err) = configuration::save_profile(
                profile,
                app_state,
                cfg_state,
                key_bindings,
                repo,
                screen_manager,
            ) {
                notification_service.set_error(AppErrorState::error(err.to_string()));
            }
            Ok(ScreenState::Refresh)
        }
        ScreenState::SaveProfileAndClose(profile) => {
            if let Err(err) = configuration::save_profile(
                profile,
                app_state,
                cfg_state,
                key_bindings,
                repo,
                screen_manager,
            ) {
                notification_service.set_error(AppErrorState::error(err.to_string()));
                return Ok(ScreenState::Refresh);
            }
            if close_on_save {
                Ok(ScreenState::Close)
            } else {
                Ok(ScreenState::Refresh)
            }
        }
        ScreenState::ApplyTheme(theme_id) => {
            if let Err(err) = configuration::save_theme(theme_id.as_str(), cfg_state) {
                notification_service.set_error(AppErrorState::error(err.to_string()));
            }
            if let Some(screen) = screen_manager.settings_themes_mut() {
                screen.set_active_theme(theme_id.as_str());
            }
            Ok(ScreenState::Refresh)
        }
        ScreenState::SaveCustomTheme(theme) => {
            if let Err(err) = configuration::save_custom_theme(theme, cfg_state) {
                notification_service.set_error(AppErrorState::error(err.to_string()));
                return Ok(ScreenState::Refresh);
            }
            let theme_id = configuration::current_theme_id(cfg_state).to_string();
            if let Some(screen) = screen_manager.settings_themes_mut() {
                screen.set_active_theme(theme_id.as_str());
            }
            screen_manager.invalidate(ScreenType::SettingsThemes);
            Ok(ScreenState::Refresh)
        }
        ScreenState::SaveCustomThemeAndClose(theme) => {
            if let Err(err) = configuration::save_custom_theme(theme, cfg_state) {
                notification_service.set_error(AppErrorState::error(err.to_string()));
                return Ok(ScreenState::Refresh);
            }
            screen_manager.invalidate(ScreenType::SettingsThemes);
            if close_on_save {
                Ok(ScreenState::Close)
            } else {
                Ok(ScreenState::Refresh)
            }
        }
        other => Ok(other),
    }
}
