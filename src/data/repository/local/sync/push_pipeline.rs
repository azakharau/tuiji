use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::data::repository::sqlite::OutboxRecord;

const OUTBOX_BATCH_SIZE: i64 = 20;
const MAX_OUTBOX_ATTEMPTS: i64 = 5;

pub(super) struct PushPipeline<'a> {
    cache: &'a SqliteRepository,
    remote: &'a JiraRepository,
}

impl<'a> PushPipeline<'a> {
    pub(super) fn new(cache: &'a SqliteRepository, remote: &'a JiraRepository) -> Self {
        Self { cache, remote }
    }

    pub(super) async fn run(&self) -> Result<()> {
        let items = self.cache.fetch_pending_outbox(OUTBOX_BATCH_SIZE).await?;
        if items.is_empty() {
            return Ok(());
        }

        let mut failed = 0usize;
        for item in items {
            if self.process_item(&item).await?.is_failed() {
                failed += 1;
            }
        }

        if failed > 0 {
            return Err(eyre!(
                "push pipeline completed with {failed} failed outbox item(s)"
            ));
        }
        Ok(())
    }

    async fn process_item(&self, item: &OutboxRecord) -> Result<ProcessOutcome> {
        if item.attempts >= MAX_OUTBOX_ATTEMPTS {
            self.cache
                .mark_outbox_failed(item.id.as_str(), "max attempts reached")
                .await?;
            return Ok(ProcessOutcome::Failed);
        }

        if self
            .cache
            .outbox_entity_conflict(item.entity_type.as_str(), item.entity_id.as_str())
            .await?
        {
            self.cache
                .mark_outbox_conflict(item.id.as_str(), Some("conflict"))
                .await?;
            return Ok(ProcessOutcome::Failed);
        }

        self.cache.mark_outbox_processing(item.id.as_str()).await?;

        match self
            .remote
            .apply_outbox_change(
                item.entity_type.as_str(),
                item.entity_id.as_str(),
                item.change_set.as_str(),
            )
            .await
        {
            Ok(()) => {
                self.cache
                    .clear_outbox_entity_dirty(item.entity_type.as_str(), item.entity_id.as_str())
                    .await?;
                self.cache.mark_outbox_done(item.id.as_str()).await?;
                Ok(ProcessOutcome::Done)
            }
            Err(err) => {
                self.cache
                    .mark_outbox_failed(item.id.as_str(), err.to_string().as_str())
                    .await?;
                Ok(ProcessOutcome::Failed)
            }
        }
    }
}

enum ProcessOutcome {
    Done,
    Failed,
}

impl ProcessOutcome {
    fn is_failed(&self) -> bool {
        matches!(self, ProcessOutcome::Failed)
    }
}
