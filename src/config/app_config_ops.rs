use std::path::{Path, PathBuf};

use super::*;

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let cfg_path = resolve_cfg_path();
        let content = std::fs::read_to_string(&cfg_path).map_err(|e| ConfigError::Io {
            source: e,
            path: cfg_path.clone(),
        })?;

        let cfg =
            toml::from_str::<AppConfig>(&content).map_err(|e| ConfigError::DeserializeToml {
                source: e,
                path: cfg_path.clone(),
            })?;
        Ok(env_override_config(cfg))
    }

    pub fn load_state() -> AppConfigState {
        match Self::load() {
            Ok(cfg) => AppConfigState::Loaded(Box::new(cfg)),
            Err(err) => AppConfigState::Missing(err),
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let cfg_path = resolve_cfg_path();
        if let Some(parent) = cfg_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
                source: e,
                path: cfg_path.clone(),
            })?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeToml {
            source: e,
            path: cfg_path.clone(),
        })?;

        std::fs::write(&cfg_path, content).map_err(|e| ConfigError::Io {
            source: e,
            path: cfg_path.clone(),
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let _ = std::fs::set_permissions(&cfg_path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    pub fn active_profile(&self) -> Option<&ProfileConfig> {
        if let Some(id) = &self.active_profile_id
            && let Some(profile) = self.profiles.iter().find(|p| &p.id == id)
        {
            return Some(profile);
        }
        self.profiles.first()
    }

    pub fn active_profile_mut(&mut self) -> Option<&mut ProfileConfig> {
        if let Some(id) = &self.active_profile_id
            && let Some(pos) = self.profiles.iter().position(|p| &p.id == id)
        {
            return self.profiles.get_mut(pos);
        }
        self.profiles.first_mut()
    }

    pub fn upsert_profile(&mut self, profile: ProfileConfig) {
        match self.profiles.iter().position(|p| p.id == profile.id) {
            Some(idx) => self.profiles[idx] = profile,
            None => self.profiles.push(profile),
        }
    }

    pub fn remove_profile(&mut self, id: &str) -> bool {
        let Some(pos) = self.profiles.iter().position(|p| p.id == id) else {
            return false;
        };
        self.profiles.remove(pos);
        if self.active_profile_id.as_deref() == Some(id) {
            self.active_profile_id = self.profiles.first().map(|p| p.id.clone());
        }
        true
    }

    pub fn set_active_profile(&mut self, id: &str) {
        self.active_profile_id = Some(id.to_string());
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            profiles: Vec::new(),
            active_profile_id: None,
            ui: UiConfig {
                theme: UiConfig::default_theme(),
                custom_themes: Vec::new(),
                screen_cache_ttl_seconds: UiConfig::default_screen_cache_ttl_seconds(),
                notification_ttl_seconds: UiConfig::default_notification_ttl_seconds(),
                notification_stack_limit: UiConfig::default_notification_stack_limit(),
                error_ttl_seconds: UiConfig::default_error_ttl_seconds(),
            },
            sync: SyncConfig::default(),
            keybindings: KeyBindingsConfig::default(),
        }
    }
}

pub fn resolve_config_dir() -> PathBuf {
    if let Ok(path) = std::env::var(format!("{}CFG_FILE_PATH", ENV_PREFIX)) {
        let path = PathBuf::from(path);
        return path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
    }

    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config_home).join(CFG_DIR);
    }
    std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join(CFG_DIR)
}

fn resolve_cfg_path() -> PathBuf {
    if let Ok(path) = std::env::var(format!("{}CFG_FILE_PATH", ENV_PREFIX)) {
        return PathBuf::from(path);
    }

    resolve_config_dir().join("config.toml")
}

fn env_override_config(mut config: AppConfig) -> AppConfig {
    if let Some(profile) = config.active_profile_mut() {
        profile.jira.env_override();
    }
    config.sync.env_override();
    config.ui.env_override();
    config
}
