use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{app::event::AppEvent, config::AppConfig, data::SqliteRepository};

/// Periodic workers are disabled in favor of WorkerController queue.
pub fn spawn_sync_pull_worker(
    _tx: UnboundedSender<AppEvent>,
    _cfg: AppConfig,
    _db: SqliteRepository,
) -> JoinHandle<()> {
    tokio::spawn(async move {})
}

/// Periodic workers are disabled in favor of WorkerController queue.
pub fn spawn_sync_push_worker(
    _tx: UnboundedSender<AppEvent>,
    _cfg: AppConfig,
    _db: SqliteRepository,
) -> JoinHandle<()> {
    tokio::spawn(async move {})
}
