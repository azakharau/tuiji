use std::time::Duration;

use color_eyre::eyre::Result;

use crate::app::App;

mod dispatch;
mod runtime;
mod worker;

use dispatch::dispatch_event;
use runtime::{spawn_input_listener, spawn_tick};
use worker::start_next_job;

pub async fn run(app: &mut App) -> Result<()> {
    app.init_db().await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _input = spawn_input_listener(tx.clone());
    let _tick = spawn_tick(tx.clone(), Duration::from_millis(250));

    // Initial paint.
    app.init_start_screen();
    app.render().await?;

    while let Some(event) = rx.recv().await {
        let dispatch = dispatch_event(app, event).await?;
        if dispatch.quit {
            break;
        }
        let render_requested = dispatch.render_requested;

        if !app.worker_controller.is_paused() {
            start_next_job(app, &tx);
        }

        if render_requested {
            app.render().await?;
        }
    }

    Ok(())
}
