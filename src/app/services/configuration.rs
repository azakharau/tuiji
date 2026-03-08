use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};

use crate::{
    app::{
        AppState, ProfileEditorIntent, key_handlers::KeyBindings, screen_manager::ScreenManager,
        state::ScreenType,
    },
    config::{AppConfig, AppConfigState, CustomThemeConfig, ProfileConfig, SyncMode},
    data::RepositoryHub,
};

pub fn cfg_or_default(cfg_state: &AppConfigState) -> AppConfig {
    match cfg_state {
        AppConfigState::Loaded(cfg) => cfg.as_ref().clone(),
        AppConfigState::Missing(_) => AppConfig::default(),
    }
}

pub fn save_profile(
    profile: ProfileConfig,
    state: &mut AppState,
    cfg_state: &mut AppConfigState,
    key_bindings: &mut Arc<KeyBindings>,
    repo: &mut Option<Arc<RepositoryHub>>,
    screen_manager: &mut ScreenManager,
) -> Result<()> {
    let mut cfg = cfg_or_default(cfg_state);
    let profile_id = profile.id.clone();
    let name_lower = profile.name.to_lowercase();
    if cfg
        .profiles
        .iter()
        .any(|p| p.id != profile_id && p.name.to_lowercase() == name_lower)
    {
        return Err(eyre!(format!(
            "A profile named \"{}\" already exists",
            profile.name
        )));
    }
    cfg.upsert_profile(profile);
    cfg.set_active_profile(&profile_id);
    save_config(cfg, cfg_state, key_bindings, repo, screen_manager)?;
    state.profile_editor = Some(ProfileEditorIntent::Edit(profile_id.clone()));
    if let Some(screen) = screen_manager.profile_creation_mut() {
        screen.set_profile_id(profile_id);
    }
    Ok(())
}

pub fn save_config(
    cfg: AppConfig,
    cfg_state: &mut AppConfigState,
    key_bindings: &mut Arc<KeyBindings>,
    repo: &mut Option<Arc<RepositoryHub>>,
    screen_manager: &mut ScreenManager,
) -> Result<()> {
    cfg.save()?;
    *key_bindings = Arc::new(KeyBindings::from_config(&cfg.keybindings));
    let refreshed_repo = if let Some(current_repo) = repo.as_ref() {
        Arc::new(current_repo.with_profile(cfg.active_profile())?)
    } else {
        return Err(eyre!(
            "Repository not initialized: cannot refresh active profile"
        ));
    };
    *repo = Some(refreshed_repo);
    *cfg_state = AppConfigState::Loaded(Box::new(cfg));
    let selected_profile_id = screen_manager
        .profiles_mut()
        .and_then(|screen| screen.selected_profile_id().map(ToString::to_string));
    if let Some(screen) = screen_manager.profiles_mut()
        && let AppConfigState::Loaded(cfg) = &cfg_state
    {
        screen.refresh(
            &cfg.profiles,
            cfg.active_profile_id.as_deref(),
            selected_profile_id.as_deref(),
        );
    }
    screen_manager.invalidate_many(&[ScreenType::Home, ScreenType::CurrentSprint]);
    Ok(())
}

pub fn save_theme(theme_id: &str, cfg_state: &mut AppConfigState) -> Result<()> {
    let mut cfg = cfg_or_default(cfg_state);
    cfg.ui.set_theme(theme_id);
    cfg.save()?;
    *cfg_state = AppConfigState::Loaded(Box::new(cfg));
    Ok(())
}

pub fn save_custom_theme(theme: CustomThemeConfig, cfg_state: &mut AppConfigState) -> Result<()> {
    let mut theme = theme;
    theme.id = theme.id.to_lowercase();
    let mut cfg = cfg_or_default(cfg_state);
    if crate::ui::theme::ThemeRegistry::is_builtin_id(theme.id.as_str()) {
        return Err(eyre!(
            "Theme id \"{}\" conflicts with built-in theme",
            theme.id
        ));
    }
    if let Some(existing) = cfg.ui.custom_themes.iter_mut().find(|t| t.id == theme.id) {
        *existing = theme.clone();
    } else {
        cfg.ui.custom_themes.push(theme.clone());
    }
    cfg.ui.set_theme(theme.id.as_str());
    cfg.save()?;
    *cfg_state = AppConfigState::Loaded(Box::new(cfg));
    Ok(())
}

pub fn current_theme_id(cfg_state: &AppConfigState) -> &str {
    match cfg_state {
        AppConfigState::Loaded(cfg) => cfg.ui.theme.as_str(),
        AppConfigState::Missing(_) => "default",
    }
}

pub fn switch_to_offline(
    cfg_state: &mut AppConfigState,
    key_bindings: &mut Arc<KeyBindings>,
    repo: &mut Option<Arc<RepositoryHub>>,
    screen_manager: &mut ScreenManager,
) -> Result<Option<String>> {
    let mut cfg = cfg_or_default(cfg_state);
    let Some(profile) = cfg.active_profile_mut() else {
        return Ok(None);
    };
    profile.set_sync_mode(SyncMode::Cache);
    let name = profile.name.clone();
    save_config(cfg, cfg_state, key_bindings, repo, screen_manager)?;
    Ok(Some(name))
}
