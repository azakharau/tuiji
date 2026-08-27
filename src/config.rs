use serde::{Deserialize, Serialize};

use crate::ConfigError;

mod app_config_ops;
mod env_overrides;
mod keybinding_defaults;
mod keybindings;
mod profile;
pub mod sync;
mod ui;
pub use app_config_ops::resolve_config_dir;
pub use keybindings::{BindingAction, KeyBindingConfig, KeyBindingsConfig};
pub use profile::{JiraConfig, ProfileConfig};
pub use sync::SyncConfig;
pub use ui::{CustomThemeConfig, ThemePaletteConfig, UiConfig};

const ENV_PREFIX: &str = "TUIJI_";
const CFG_DIR: &str = "tuiji";

#[derive(Debug)]
pub enum AppConfigState {
    Loaded(Box<AppConfig>),
    Missing(ConfigError),
}

impl AppConfigState {
    pub fn is_loaded(&self) -> bool {
        matches!(self, AppConfigState::Loaded(_))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub profiles: Vec<ProfileConfig>,
    #[serde(default)]
    pub active_profile_id: Option<String>,
    pub ui: UiConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default, skip_serializing_if = "is_keybindings_default")]
    pub keybindings: KeyBindingsConfig,
}

fn is_keybindings_default(kb: &KeyBindingsConfig) -> bool {
    // Skip serializing keybindings if it equals the default
    kb == &KeyBindingsConfig::default()
}

// Key bindings are configured via [keybindings] with vim-style defaults.

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
        let config = AppConfig {
            profiles: vec![ProfileConfig {
                id: "test".to_string(),
                name: "Test".to_string(),
                jira: JiraConfig {
                    base_url: "http://test.com".to_string(),
                    username: "test".to_string(),
                    api_token: "token".to_string(),
                    api_token_command: None,
                },
            }],
            ..AppConfig::default()
        };

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
