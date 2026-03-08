use serde::{Deserialize, Serialize};

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
