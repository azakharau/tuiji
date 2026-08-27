use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub jira: JiraConfig,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct JiraConfig {
    pub base_url: String,
    pub username: String,
    pub api_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_token_command: Option<String>,
}

impl JiraConfig {
    pub fn resolve_token(&self) -> color_eyre::Result<String> {
        if !self.api_token.trim().is_empty() {
            return Ok(self.api_token.clone());
        }

        if let Some(command) = &self.api_token_command {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(color_eyre::eyre::eyre!(
                    "api_token_command failed: {}",
                    stderr.trim_end()
                ));
            }

            let token = String::from_utf8_lossy(&output.stdout);
            let token = token.trim_end();
            if token.is_empty() {
                return Err(color_eyre::eyre::eyre!(
                    "api_token_command returned empty output"
                ));
            }
            return Ok(token.to_string());
        }

        Err(color_eyre::eyre::eyre!(
            "No Jira API token: set api_token, api_token_command, or TUIJI_JIRA_API_TOKEN"
        ))
    }
}

impl std::fmt::Debug for JiraConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JiraConfig")
            .field("base_url", &self.base_url)
            .field("username", &self.username)
            .field("api_token", &"[redacted]")
            .field(
                "api_token_command",
                &self.api_token_command.as_ref().map(|_| "[redacted]"),
            )
            .finish()
    }
}
