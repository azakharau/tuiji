use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::data::{
    IssueSnapshot,
    model::{OutboxChange, OutboxCommand},
    repository::CommandRepository,
};

impl RepositoryHub {
    pub(super) async fn resolve_conflict_use_local_impl(&self, key: &str) -> Result<()> {
        let Some(issue) = self.cache.fetch_issue(key).await? else {
            return Err(eyre!("Issue {} not found", key));
        };

        let mut fields = serde_json::Map::new();
        fields.insert(
            "summary".to_string(),
            serde_json::Value::String(issue.summary),
        );
        if let Some(description) = issue.description {
            fields.insert(
                "description".to_string(),
                serde_json::to_value(gouqi::AdfDocument::from_text(description))?,
            );
        }
        if !issue.priority.is_empty() {
            fields.insert(
                "priority".to_string(),
                serde_json::json!({ "name": issue.priority }),
            );
        }

        let change = OutboxChange::Fields { fields };
        let command = OutboxCommand::issue(key, serde_json::to_string(&change)?);

        self.cache.resolve_issue_use_local(key).await?;
        self.cache.enqueue_outbox(command).await
    }

    pub(super) async fn resolve_conflict_use_remote_impl(&self, key: &str) -> Result<()> {
        let Some(issue) = self.cache.fetch_issue(key).await? else {
            return Err(eyre!("Issue {} not found", key));
        };
        let snapshot = issue
            .remote_snapshot
            .ok_or_else(|| eyre!("Remote snapshot missing for issue {}", key))?;
        let remote: IssueSnapshot = serde_json::from_str(snapshot.as_str())?;
        self.cache.resolve_issue_use_remote(key, &remote).await
    }
}
