use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::{
    client::jira::write::WriteError,
    data::{model::OutboxChange, repository::sqlite::OutboxRecord},
};

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
                if item.entity_type == "comment" && item.entity_id.starts_with("local:") {
                    self.cache.delete_comment(item.entity_id.as_str()).await?;
                } else {
                    self.cache
                        .clear_outbox_entity_dirty(
                            item.entity_type.as_str(),
                            item.entity_id.as_str(),
                        )
                        .await?;
                }
                self.cache.mark_outbox_done(item.id.as_str()).await?;
                Ok(ProcessOutcome::Done)
            }
            Err(WriteError::Conflict(msg)) => {
                let change = serde_json::from_str::<OutboxChange>(item.change_set.as_str())?;
                let issue_key = match &change {
                    OutboxChange::Comment { issue_key, .. } => issue_key.as_str(),
                    _ => item.entity_id.as_str(),
                };
                self.cache
                    .mark_outbox_conflict(item.id.as_str(), Some(&msg))
                    .await?;
                self.cache.set_issue_conflict(issue_key).await?;
                Ok(ProcessOutcome::Failed)
            }
            Err(WriteError::Permanent(msg)) => {
                self.cache
                    .mark_outbox_failed(item.id.as_str(), &msg)
                    .await?;
                Ok(ProcessOutcome::Failed)
            }
            Err(WriteError::Transient(msg)) => {
                self.cache
                    .mark_outbox_pending(item.id.as_str(), &msg)
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
        match self {
            ProcessOutcome::Done => false,
            ProcessOutcome::Failed => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::JiraConfig,
        data::{
            model::OutboxCommand,
            repository::{CommandRepository, sqlite::SqliteRepositoryConfig},
        },
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    struct Harness {
        cache: SqliteRepository,
        remote: JiraRepository,
        db_path: std::path::PathBuf,
    }

    impl Harness {
        async fn new(base_url: &str) -> Self {
            let db_path =
                std::env::temp_dir().join(format!("tuiji-push-{}.db", uuid::Uuid::now_v7()));
            let cache = SqliteRepository::connect(
                SqliteRepositoryConfig {
                    db_path: db_path.clone(),
                },
                "test-profile".to_string(),
            )
            .await
            .unwrap();
            let remote = JiraRepository::new(&JiraConfig {
                base_url: base_url.to_string(),
                username: "user@example.com".to_string(),
                api_token: "token".to_string(),
                api_token_command: None,
            })
            .unwrap();
            Self {
                cache,
                remote,
                db_path,
            }
        }

        async fn enqueue_summary_edit(&self, key: &str) {
            let change = OutboxChange::Fields {
                fields: serde_json::Map::from_iter([(
                    "summary".to_string(),
                    serde_json::json!("Updated summary"),
                )]),
            };
            self.cache
                .enqueue_outbox(OutboxCommand::issue(
                    key,
                    serde_json::to_string(&change).unwrap(),
                ))
                .await
                .unwrap();
        }

        async fn push(&self) -> Result<()> {
            PushPipeline::new(&self.cache, &self.remote).run().await
        }

        async fn pending(&self) -> Vec<OutboxRecord> {
            self.cache
                .fetch_pending_outbox(OUTBOX_BATCH_SIZE)
                .await
                .unwrap()
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.db_path);
        }
    }

    async fn jira_returning(status: u16, body: Option<serde_json::Value>) -> MockServer {
        let server = MockServer::start().await;
        let response = match body {
            Some(body) => ResponseTemplate::new(status).set_body_json(body),
            None => ResponseTemplate::new(status),
        };
        Mock::given(method("PUT"))
            .and(path("/rest/api/3/issue/TEST-1"))
            .respond_with(response)
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn transient_remote_failure_should_requeue_the_change_and_consume_one_attempt() {
        let server = jira_returning(503, None).await;
        let harness = Harness::new(&server.uri()).await;
        harness.enqueue_summary_edit("TEST-1").await;

        assert!(
            harness.push().await.is_err(),
            "a failed push must be reported to the caller"
        );

        let pending = harness.pending().await;
        assert_eq!(
            pending.len(),
            1,
            "a transient Jira failure must leave the queued write in place"
        );
        assert_eq!(
            pending[0].attempts, 1,
            "the attempt must be consumed so the retry budget can run out"
        );
    }

    #[tokio::test]
    async fn permanent_remote_rejection_should_stop_retrying_the_change() {
        let server = jira_returning(
            400,
            Some(serde_json::json!({ "errors": { "summary": "Field 'summary' cannot be set" } })),
        )
        .await;
        let harness = Harness::new(&server.uri()).await;
        harness.enqueue_summary_edit("TEST-1").await;

        assert!(harness.push().await.is_err());

        assert!(
            harness.pending().await.is_empty(),
            "a rejected write must not be retried on every sync"
        );
        assert_eq!(
            harness.cache.requeue_failed_outbox().await.unwrap(),
            1,
            "the change must be parked as failed, so an explicit retry can revive it"
        );
    }

    #[tokio::test]
    async fn conflicting_remote_state_should_park_the_change_outside_the_failed_queue() {
        let server = jira_returning(409, None).await;
        let harness = Harness::new(&server.uri()).await;
        harness.enqueue_summary_edit("TEST-1").await;

        assert!(harness.push().await.is_err());

        assert!(
            harness.pending().await.is_empty(),
            "a conflict must stop the automatic push"
        );
        assert_eq!(
            harness.cache.requeue_failed_outbox().await.unwrap(),
            0,
            "a conflict is resolved on the conflicts screen, not by the failed-outbox retry"
        );
    }

    #[tokio::test]
    async fn exhausted_attempts_should_stop_the_retry_loop() {
        let server = jira_returning(503, None).await;
        let harness = Harness::new(&server.uri()).await;
        harness.enqueue_summary_edit("TEST-1").await;

        for _ in 0..MAX_OUTBOX_ATTEMPTS {
            assert!(harness.push().await.is_err());
        }
        assert_eq!(
            harness.pending().await.first().map(|item| item.attempts),
            Some(MAX_OUTBOX_ATTEMPTS),
            "every push must consume exactly one attempt"
        );

        assert!(harness.push().await.is_err());

        assert!(
            harness.pending().await.is_empty(),
            "the attempt ceiling must stop the endless retry"
        );
        assert_eq!(harness.cache.requeue_failed_outbox().await.unwrap(), 1);
    }
}
