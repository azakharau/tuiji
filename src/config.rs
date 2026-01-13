use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ConfigError;

const ENV_PREFIX: &str = "TUIJI_";
const CFG_DIR: &str = "tuiji";

#[derive(Debug)]
pub enum AppConfigState {
    Loaded(AppConfig),
    Missing(ConfigError),
}

impl AppConfigState {
    pub fn is_loaded(&self) -> bool {
        matches!(self, AppConfigState::Loaded(_))
    }

    /// Returns the loaded config if available, or an error if not loaded
    pub fn as_loaded(&self) -> Result<&AppConfig, &ConfigError> {
        match self {
            AppConfigState::Loaded(cfg) => Ok(cfg),
            AppConfigState::Missing(err) => Err(err),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub profiles: Vec<ProfileConfig>,
    #[serde(default)]
    pub active_profile_id: Option<String>,
    pub ui: UiConfig,
    #[serde(default, skip_serializing_if = "is_keybindings_default")]
    pub keybindings: KeyBindingsConfig,
}

fn is_keybindings_default(kb: &KeyBindingsConfig) -> bool {
    // Skip serializing keybindings if it equals the default
    kb == &KeyBindingsConfig::default()
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let cfg_path = resolve_cfg_path();
        let content = std::fs::read_to_string(&cfg_path).map_err(|e| ConfigError::Io {
            source: e,
            path: cfg_path.clone(),
        })?;

        let value =
            toml::from_str::<toml::Value>(&content).map_err(|e| ConfigError::DeserializeToml {
                source: e,
                path: cfg_path.clone(),
            })?;

        let cfg = if value.get("profiles").is_some() {
            toml::from_str::<AppConfig>(&content).map_err(|e| ConfigError::DeserializeToml {
                source: e,
                path: cfg_path.clone(),
            })?
        } else {
            let legacy = toml::from_str::<LegacyAppConfig>(&content).map_err(|e| {
                ConfigError::DeserializeToml {
                    source: e,
                    path: cfg_path.clone(),
                }
            })?;
            AppConfig::from_legacy(legacy)
        };

        Ok(env_override_config(cfg))
    }

    pub fn load_state() -> AppConfigState {
        match Self::load() {
            Ok(cfg) => AppConfigState::Loaded(cfg),
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
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub jira: JiraConfig,
    #[serde(default)]
    pub sync_mode: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SyncMode {
    Cache,
    Online,
}

impl SyncMode {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncMode::Cache => "cache",
            SyncMode::Online => "online",
        }
    }

    pub fn from_opt_str(value: Option<&str>) -> Self {
        match value {
            Some("online") => SyncMode::Online,
            _ => SyncMode::Cache,
        }
    }
}

impl ProfileConfig {
    pub fn sync_mode(&self) -> SyncMode {
        SyncMode::from_opt_str(self.sync_mode.as_deref())
    }

    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.sync_mode = Some(mode.as_str().to_string());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
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

// Key bindings are configured via [keybindings] with vim-style defaults.

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UiConfig {
    #[serde(default = "UiConfig::default_theme")]
    pub theme: String,
    #[serde(default)]
    pub custom_themes: Vec<CustomThemeConfig>,
    #[serde(default = "UiConfig::default_screen_cache_ttl_seconds")]
    pub screen_cache_ttl_seconds: u64,
    #[serde(default = "UiConfig::default_notification_ttl_seconds")]
    pub notification_ttl_seconds: u64,
    #[serde(default = "UiConfig::default_notification_stack_limit")]
    pub notification_stack_limit: usize,
    #[serde(default = "UiConfig::default_error_ttl_seconds")]
    pub error_ttl_seconds: u64,
}

impl UiConfig {
    pub fn set_theme(&mut self, theme: &str) {
        self.theme = theme.to_string();
    }

    pub fn env_override(&mut self) {
        if let Ok(theme) = std::env::var(format!("{}UI_THEME", ENV_PREFIX)) {
            self.theme = theme;
        }
        if let Ok(ttl) = std::env::var(format!("{}UI_NOTIFICATION_TTL_SECONDS", ENV_PREFIX)) {
            if let Ok(value) = ttl.parse::<u64>() {
                self.notification_ttl_seconds = value;
            }
        }
        if let Ok(limit) = std::env::var(format!("{}UI_NOTIFICATION_STACK_LIMIT", ENV_PREFIX)) {
            if let Ok(value) = limit.parse::<usize>() {
                self.notification_stack_limit = value;
            }
        }
        if let Ok(ttl) = std::env::var(format!("{}UI_ERROR_TTL_SECONDS", ENV_PREFIX)) {
            if let Ok(value) = ttl.parse::<u64>() {
                self.error_ttl_seconds = value;
            }
        }
    }
}

impl UiConfig {
    fn default_theme() -> String {
        "default".to_string()
    }

    fn default_screen_cache_ttl_seconds() -> u64 {
        60
    }

    fn default_notification_ttl_seconds() -> u64 {
        5
    }

    fn default_notification_stack_limit() -> usize {
        5
    }

    fn default_error_ttl_seconds() -> u64 {
        6
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
            keybindings: KeyBindingsConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ThemePaletteConfig {
    pub background: String,
    pub text: String,
    pub accent: String,
    pub selection: String,
    pub border: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    pub success: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CustomThemeConfig {
    pub id: String,
    pub name: String,
    pub palette: ThemePaletteConfig,
}

pub fn resolve_config_dir() -> PathBuf {
    if let Ok(path) = std::env::var(format!("{}CFG_FILE_PATH", ENV_PREFIX)) {
        let path = PathBuf::from(path);
        return path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
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
    config.ui.env_override();
    config
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
struct LegacyAppConfig {
    pub jira: JiraConfig,
    pub ui: UiConfig,
}

impl AppConfig {
    pub fn active_profile(&self) -> Option<&ProfileConfig> {
        if let Some(id) = &self.active_profile_id {
            if let Some(profile) = self.profiles.iter().find(|p| &p.id == id) {
                return Some(profile);
            }
        }
        self.profiles.first()
    }

    pub fn active_profile_mut(&mut self) -> Option<&mut ProfileConfig> {
        if let Some(id) = &self.active_profile_id {
            if let Some(pos) = self.profiles.iter().position(|p| &p.id == id) {
                return self.profiles.get_mut(pos);
            }
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

    fn from_legacy(legacy: LegacyAppConfig) -> Self {
        let id = uuid::Uuid::now_v7().to_string();
        let profile = ProfileConfig {
            id: id.clone(),
            name: "Default".to_string(),
            jira: legacy.jira,
            sync_mode: None,
        };
        AppConfig {
            profiles: vec![profile],
            active_profile_id: Some(id),
            ui: legacy.ui,
            keybindings: KeyBindingsConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct KeyBindingsConfig {
    #[serde(default)]
    pub global: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub home: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub board_selection: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub current_sprint: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profile_creation: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub my_issues: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_issues: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub new_issue: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sync_status: Vec<KeyBindingConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issue_detail: Vec<KeyBindingConfig>,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        Self {
            global: vec![
                KeyBindingConfig::new(BindingAction::Quit, "q"),
                KeyBindingConfig::new(BindingAction::Refresh, "r"),
                KeyBindingConfig::new(BindingAction::Confirm, "<enter>"),
                KeyBindingConfig::new(BindingAction::GoHome, "gh"),
                KeyBindingConfig::new(BindingAction::OpenBoards, "b"),
                KeyBindingConfig::new(BindingAction::OpenInBrowser, "o"),
                KeyBindingConfig::new(BindingAction::OpenSettings, ","),
            ],
            home: vec![
                KeyBindingConfig::new(BindingAction::Quit, "q"),
                KeyBindingConfig::new(BindingAction::OpenCurrentSprint, "c"),
                KeyBindingConfig::new(BindingAction::OpenMyIssues, "i"),
                KeyBindingConfig::new(BindingAction::OpenSearchIssues, "s"),
                KeyBindingConfig::new(BindingAction::OpenNewIssue, "n"),
                KeyBindingConfig::new(BindingAction::OpenBoards, "b"),
                KeyBindingConfig::new(BindingAction::OpenSyncStatus, "t"),
                KeyBindingConfig::new(BindingAction::OpenSettings, ","),
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
            ],
            board_selection: vec![
                KeyBindingConfig::new(BindingAction::Quit, "q"),
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
            ],
            current_sprint: vec![
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveLeft, "h"),
                KeyBindingConfig::new(BindingAction::MoveLeft, "<left>"),
                KeyBindingConfig::new(BindingAction::MoveRight, "l"),
                KeyBindingConfig::new(BindingAction::MoveRight, "<right>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
            ],
            profile_creation: vec![
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveLeft, "h"),
                KeyBindingConfig::new(BindingAction::MoveLeft, "<left>"),
                KeyBindingConfig::new(BindingAction::MoveRight, "l"),
                KeyBindingConfig::new(BindingAction::MoveRight, "<right>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
                KeyBindingConfig::new(BindingAction::MoveLineStart, "0"),
                KeyBindingConfig::new(BindingAction::MoveLineStart, "^"),
                KeyBindingConfig::new(BindingAction::MoveLineEnd, "$"),
                KeyBindingConfig::new(BindingAction::MoveWordForward, "w"),
                KeyBindingConfig::new(BindingAction::MoveWordForward, "W"),
                KeyBindingConfig::new(BindingAction::MoveWordBackward, "b"),
                KeyBindingConfig::new(BindingAction::MoveWordBackward, "B"),
                KeyBindingConfig::new(BindingAction::MoveWordEnd, "e"),
                KeyBindingConfig::new(BindingAction::MoveWordEnd, "E"),
                KeyBindingConfig::new(BindingAction::EnterInsertBefore, "i"),
                KeyBindingConfig::new(BindingAction::EnterInsertAfter, "a"),
                KeyBindingConfig::new(BindingAction::EnterInsertLineStart, "I"),
                KeyBindingConfig::new(BindingAction::EnterInsertLineEnd, "A"),
            ],
            profiles: vec![
                KeyBindingConfig::new(BindingAction::Quit, "q"),
                KeyBindingConfig::new(BindingAction::EditProfile, "e"),
                KeyBindingConfig::new(BindingAction::DeleteProfile, "d"),
                KeyBindingConfig::new(BindingAction::NewProfile, "n"),
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
            ],
            settings: vec![
                KeyBindingConfig::new(BindingAction::Confirm, "<enter>"),
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
            ],
            my_issues: Vec::new(),
            search_issues: Vec::new(),
            conflicts: vec![
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
                KeyBindingConfig::new(BindingAction::ResolveConflictLocal, "l"),
                KeyBindingConfig::new(BindingAction::ResolveConflictRemote, "j"),
            ],
            sync_status: vec![
                KeyBindingConfig::new(BindingAction::SyncNow, "s"),
                KeyBindingConfig::new(BindingAction::SyncPause, "p"),
                KeyBindingConfig::new(BindingAction::SyncRetry, "t"),
                KeyBindingConfig::new(BindingAction::SyncResume, "u"),
                KeyBindingConfig::new(BindingAction::FilterAll, "A"),
                KeyBindingConfig::new(BindingAction::FilterPull, "P"),
                KeyBindingConfig::new(BindingAction::FilterPush, "U"),
            ],
            issue_detail: vec![
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
                KeyBindingConfig::new(BindingAction::PageUp, "<pageup>"),
                KeyBindingConfig::new(BindingAction::PageDown, "<pagedown>"),
                KeyBindingConfig::new(BindingAction::Refresh, "r"),
                KeyBindingConfig::new(BindingAction::OpenInBrowser, "o"),
            ],
            new_issue: vec![
                KeyBindingConfig::new(BindingAction::MoveUp, "k"),
                KeyBindingConfig::new(BindingAction::MoveUp, "<up>"),
                KeyBindingConfig::new(BindingAction::MoveDown, "j"),
                KeyBindingConfig::new(BindingAction::MoveDown, "<down>"),
                KeyBindingConfig::new(BindingAction::MoveLeft, "h"),
                KeyBindingConfig::new(BindingAction::MoveLeft, "<left>"),
                KeyBindingConfig::new(BindingAction::MoveRight, "l"),
                KeyBindingConfig::new(BindingAction::MoveRight, "<right>"),
                KeyBindingConfig::new(BindingAction::MoveTop, "gg"),
                KeyBindingConfig::new(BindingAction::MoveBottom, "G"),
                KeyBindingConfig::new(BindingAction::MoveLineStart, "0"),
                KeyBindingConfig::new(BindingAction::MoveLineStart, "^"),
                KeyBindingConfig::new(BindingAction::MoveLineEnd, "$"),
                KeyBindingConfig::new(BindingAction::MoveWordForward, "w"),
                KeyBindingConfig::new(BindingAction::MoveWordForward, "W"),
                KeyBindingConfig::new(BindingAction::MoveWordBackward, "b"),
                KeyBindingConfig::new(BindingAction::MoveWordBackward, "B"),
                KeyBindingConfig::new(BindingAction::MoveWordEnd, "e"),
                KeyBindingConfig::new(BindingAction::MoveWordEnd, "E"),
                KeyBindingConfig::new(BindingAction::EnterInsertBefore, "i"),
                KeyBindingConfig::new(BindingAction::EnterInsertAfter, "a"),
                KeyBindingConfig::new(BindingAction::EnterInsertLineStart, "I"),
                KeyBindingConfig::new(BindingAction::EnterInsertLineEnd, "A"),
                KeyBindingConfig::new(BindingAction::Confirm, "<enter>"),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct KeyBindingConfig {
    pub action: BindingAction,
    pub binding: String,
}

impl KeyBindingConfig {
    fn new(action: BindingAction, binding: &str) -> Self {
        Self {
            action,
            binding: binding.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BindingAction {
    Quit,
    Refresh,
    Confirm,
    GoHome,
    OpenCurrentSprint,
    OpenMyIssues,
    OpenSearchIssues,
    OpenNewIssue,
    OpenProfiles,
    OpenBoards,
    OpenSettings,
    OpenSyncStatus,
    ResolveConflictLocal,
    ResolveConflictRemote,
    SyncNow,
    SyncPause,
    SyncRetry,
    SyncResume,
    FilterAll,
    FilterPull,
    FilterPush,
    NewProfile,
    EditProfile,
    DeleteProfile,
    OpenInBrowser,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveTop,
    MoveBottom,
    MoveLineStart,
    MoveLineEnd,
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    PageUp,
    PageDown,
    EnterInsertBefore,
    EnterInsertAfter,
    EnterInsertLineStart,
    EnterInsertLineEnd,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_keybindings_not_serialized() {
        let mut config = AppConfig::default();
        // Clear some keybinding sections to make them empty
        config.keybindings.settings = vec![];
        config.keybindings.my_issues = vec![];
        config.keybindings.search_issues = vec![];
        config.keybindings.new_issue = vec![];

        let serialized = toml::to_string_pretty(&config).unwrap();

        // Check that empty arrays are not present
        assert!(
            !serialized.contains("settings = []"),
            "settings = [] should not be serialized"
        );
        assert!(
            !serialized.contains("my_issues = []"),
            "my_issues = [] should not be serialized"
        );
        assert!(
            !serialized.contains("search_issues = []"),
            "search_issues = [] should not be serialized"
        );
        assert!(
            !serialized.contains("new_issue = []"),
            "new_issue = [] should not be serialized"
        );
    }

    #[test]
    fn test_default_keybindings_section_not_serialized() {
        let mut config = AppConfig::default();
        // Set profiles to avoid empty config
        config.profiles = vec![ProfileConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            jira: JiraConfig {
                base_url: "http://test.com".to_string(),
                username: "test".to_string(),
                api_token: "token".to_string(),
            },
            sync_mode: None,
        }];

        let serialized = toml::to_string_pretty(&config).unwrap();

        // If keybindings are default, the entire [keybindings] section should not be present
        assert!(
            !serialized.contains("[keybindings]"),
            "Default keybindings section should not be serialized.\nSerialized config:\n{}",
            serialized
        );
    }

    #[test]
    fn test_custom_keybindings_are_serialized() {
        let mut config = AppConfig::default();
        // Add a custom keybinding
        config.keybindings.home.push(KeyBindingConfig {
            action: BindingAction::MoveUp,
            binding: "custom".to_string(),
        });

        let serialized = toml::to_string_pretty(&config).unwrap();

        // Custom keybindings should be present
        assert!(
            serialized.contains("[[keybindings.global]]")
                || serialized.contains("[[keybindings.home]]"),
            "Custom keybindings should be serialized"
        );
        assert!(
            serialized.contains("[[keybindings.home]]"),
            "Custom home keybindings should be serialized"
        );
    }
}
