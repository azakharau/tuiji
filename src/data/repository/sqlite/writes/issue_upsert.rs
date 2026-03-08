use super::*;

impl SqliteRepository {
    pub(crate) async fn upsert_issues_impl(&self, issues: &[IssueSummary]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let now_ts = current_ts();

        for issue in issues {
            if self.should_skip_issue(&mut tx, &issue.key).await? {
                continue;
            }

            let updated_at = system_time_to_ts(issue.updated_at);
            self.upsert_issue_row(&mut tx, issue, updated_at).await?;

            let snapshot_at = updated_at.unwrap_or(now_ts);
            self.insert_issue_history_row(&mut tx, issue, snapshot_at)
                .await?;

            if self.has_dirty_comments(&mut tx, &issue.key).await? {
                continue;
            }

            self.replace_issue_comments(&mut tx, issue).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
