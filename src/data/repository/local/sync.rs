use color_eyre::Result;

use super::*;
use crate::data::repository::SyncExecutor;

mod pull;
mod push_pipeline;
mod reconcile;

#[async_trait::async_trait]
impl SyncExecutor for RepositoryHub {
    async fn sync_pull(&self) -> Result<()> {
        RepositoryHub::sync_pull(self).await
    }

    async fn sync_push(&self) -> Result<()> {
        RepositoryHub::sync_push(self).await
    }
}
