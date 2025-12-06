use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{AppError, ConfigError};

const ENV_PREFIX: &str = "TUIJI_";
const CFG_FILE_PATH: &str = "tuiji/config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub jira: JiraConfig,
    pub key_bindings: KeyBindings,
    pub ui: UiConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
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

    pub fn save(&self) -> Result<(), AppError> {
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

        std::fs::write(&cfg_path, content)
            .map_err(|e| ConfigError::Io {
                source: e,
                path: cfg_path.clone(),
            })
            .map_err(AppError::from)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraConfig {
    pub base_url: String,
    pub username: String,
    pub api_token: String,
}

impl JiraConfig {
    pub fn env_override(&mut self) {
        if let Ok(base_url) = std::env::var(format!("{}JIRA_BASE_URL", ENV_PREFIX)) {
            self.base_url = base_url;
        }
        if let Ok(username) = std::env::var(format!("{}JIRA_USERNAME", ENV_PREFIX)) {
            self.username = username;
        }
        if let Ok(api_token) = std::env::var(format!("{}JIRA_API_TOKEN", ENV_PREFIX)) {
            self.api_token = api_token;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyBindings {
    pub quit: String,
    pub next: String,
    pub previous: String,
    pub open_in_browser: String,
    pub refresh: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UiConfig {
    #[serde(default = "UiConfig::default_theme")]
    pub theme: String,
}

impl UiConfig {
    pub fn set_theme(&mut self, theme: &str) {
        self.theme = theme.to_string();
    }

    pub fn env_override(&mut self) {
        if let Ok(theme) = std::env::var(format!("{}UI_THEME", ENV_PREFIX)) {
            self.theme = theme;
        }
    }
}

impl UiConfig {
    fn default_theme() -> String {
        "dark".to_string()
    }
}

fn resolve_cfg_path() -> PathBuf {
    if let Ok(path) = std::env::var(format!("{}CFG_FILE_PATH", ENV_PREFIX)) {
        return PathBuf::from(path);
    }

    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config_home).join(CFG_FILE_PATH);
    }
    std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join(CFG_FILE_PATH)
}

fn env_override_config(mut config: AppConfig) -> AppConfig {
    config.jira.env_override();
    config.ui.env_override();
    config
}
