use super::*;

impl JiraConfig {
    pub(super) fn env_override(&mut self) {
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

impl UiConfig {
    pub(super) fn env_override(&mut self) {
        if let Ok(theme) = std::env::var(format!("{}UI_THEME", ENV_PREFIX)) {
            self.theme = theme;
        }
        if let Ok(ttl) = std::env::var(format!("{}UI_NOTIFICATION_TTL_SECONDS", ENV_PREFIX))
            && let Ok(value) = ttl.parse::<u64>()
        {
            self.notification_ttl_seconds = value;
        }
        if let Ok(limit) = std::env::var(format!("{}UI_NOTIFICATION_STACK_LIMIT", ENV_PREFIX))
            && let Ok(value) = limit.parse::<usize>()
        {
            self.notification_stack_limit = value;
        }
        if let Ok(ttl) = std::env::var(format!("{}UI_ERROR_TTL_SECONDS", ENV_PREFIX))
            && let Ok(value) = ttl.parse::<u64>()
        {
            self.error_ttl_seconds = value;
        }
    }
}
