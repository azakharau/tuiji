use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::data::repository::QueryRepository;

impl RepositoryHub {
    pub async fn sync_pull(&self) -> Result<()> {
        let Some(remote) = &self.remote else {
            self.cache
                .log_sync_event(
                    "pull",
                    "error",
                    Some("sync pull requires online mode"),
                    Some(self.profile_id.as_str()),
                )
                .await?;
            return Err(eyre!("Sync pull requires online mode"));
        };

        let board_ids = self.cache.selected_board_ids().await?;
        let result: Result<()> = async {
            for board_id in board_ids {
                let config = remote.board_config(board_id).await?;
                self.cache.upsert_board_config(board_id, &config).await?;
                self.cache
                    .replace_board_columns(board_id, &config.columns)
                    .await?;

                let issues = remote.current_sprint_issues(board_id).await?;
                for issue in &issues {
                    self.reconcile_pull_issue(issue).await?;
                }
            }
            Ok(())
        }
        .await;

        log_sync_outcome(self, "pull", result).await
    }

    pub async fn sync_push(&self) -> Result<()> {
        let Some(remote) = &self.remote else {
            self.cache
                .log_sync_event(
                    "push",
                    "error",
                    Some("sync push requires online mode"),
                    Some(self.profile_id.as_str()),
                )
                .await?;
            return Err(eyre!("Sync push requires online mode"));
        };

        let pipeline = push_pipeline::PushPipeline::new(self.cache.as_ref(), remote.as_ref());
        let result = pipeline.run().await;
        log_sync_outcome(self, "push", result).await
    }
}

async fn log_sync_outcome(repo: &RepositoryHub, direction: &str, result: Result<()>) -> Result<()> {
    match result {
        Ok(()) => {
            repo.cache
                .log_sync_event(direction, "success", None, Some(repo.profile_id.as_str()))
                .await?;
            Ok(())
        }
        Err(err) => {
            repo.cache
                .log_sync_event(
                    direction,
                    "error",
                    Some(&err.to_string()),
                    Some(repo.profile_id.as_str()),
                )
                .await?;
            Err(err)
        }
    }
}
