use serde::{Deserialize, Serialize};

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

    pub(crate) fn default_theme() -> String {
        "default".to_string()
    }

    pub(crate) fn default_screen_cache_ttl_seconds() -> u64 {
        60
    }

    pub(crate) fn default_notification_ttl_seconds() -> u64 {
        5
    }

    pub(crate) fn default_notification_stack_limit() -> usize {
        5
    }

    pub(crate) fn default_error_ttl_seconds() -> u64 {
        6
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
