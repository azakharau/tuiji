use color_eyre::eyre::Result;

use super::*;
use crate::app::{
    ActionOutcome,
    error::AppErrorState,
    event::{AppEvent, InputEvent, SystemEvent, UiEvent},
    navigation::NavigationController,
};

pub(super) struct DispatchOutcome {
    pub render_requested: bool,
    pub quit: bool,
}

pub(super) async fn dispatch_event(app: &mut App, event: AppEvent) -> Result<DispatchOutcome> {
    let mut render_requested = false;
    let mut quit = false;

    match event {
        AppEvent::Input(InputEvent::Key(key)) => {
            if app.worker_controller.is_paused() {
                render_requested = runtime::handle_sync_modal_input(app, key);
            } else {
                let action = app.handle_input(key).await?;
                let outcome = {
                    let mut nav = NavigationController::new(
                        &mut app.state,
                        &mut app.screen_stack,
                        &mut app.screen_manager,
                        &mut app.terminal,
                        &app.cfg_state,
                        &app.key_bindings,
                    );
                    nav.apply_action(action)?
                };
                app.enforce_command_mode_allowed();
                match outcome {
                    ActionOutcome::Continue { render } => render_requested |= render,
                    ActionOutcome::Quit => quit = true,
                }
            }
        }
        AppEvent::System(SystemEvent::Tick) => {
            render_requested |= app.notification_service.tick();
        }
        AppEvent::Worker(msg) => {
            render_requested |= worker::handle_worker_event(app, msg).await;
        }
        AppEvent::Ui(UiEvent::Error(err)) => {
            app.notification_service
                .set_error(AppErrorState::error(err));
            render_requested = true;
        }
    }

    Ok(DispatchOutcome {
        render_requested,
        quit,
    })
}
