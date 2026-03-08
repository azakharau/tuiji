use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle, time};

use super::*;
use crate::app::{
    event::{AppEvent, InputEvent, SystemEvent},
    input::is_question_mark,
};

pub(super) fn handle_sync_modal_input(app: &mut App, key: KeyEvent) -> bool {
    if is_question_mark(&key) {
        return true;
    }
    match key.code {
        KeyCode::Char('r') => {
            app.worker_controller.resume();
            if let Some(job) = app.worker_controller.take_last_failed_job() {
                let mut job = job;
                job.next_attempt_at = None;
                app.worker_controller.enqueue_front(job);
            }
            true
        }
        KeyCode::Char('q') => {
            app.worker_controller.stop();
            true
        }
        _ => false,
    }
}

pub(super) fn spawn_input_listener(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(event) = reader.next().await {
            match event {
                Ok(Event::Key(key)) => {
                    let _ = tx.send(AppEvent::Input(InputEvent::Key(key)));
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("event stream error: {err}");
                    break;
                }
            }
        }
    })
}

pub(super) fn spawn_tick(tx: UnboundedSender<AppEvent>, period: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = time::interval(period);
        loop {
            ticker.tick().await;
            if tx.send(AppEvent::System(SystemEvent::Tick)).is_err() {
                break;
            }
        }
    })
}
