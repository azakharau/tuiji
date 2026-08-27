use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SyncConfig {
    #[serde(default = "default_interval_seconds")]
    pub interval_seconds: u64,
}

fn default_interval_seconds() -> u64 {
    120
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_seconds: default_interval_seconds(),
        }
    }
}
