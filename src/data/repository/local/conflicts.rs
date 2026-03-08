use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::data::IssueSnapshot;

impl RepositoryHub {
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
