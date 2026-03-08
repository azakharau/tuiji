use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame};

use super::{AppRenderer, RenderState};
use crate::{
    app::{
        key_handlers::{action_hints, binding_hints_for_prefix, binding_hints_for_screen},
        overlay::{OverlayBus, OverlayItem, WhichKeyMode},
        screen_manager::ScreenManager,
    },
    config::{AppConfig, AppConfigState},
    ui::{
        context::RenderContext,
        overlays::{
            BoardRequiredModal, CommandLineModal, ErrorModal, NotificationModal, SyncErrorModal,
            WhichKeyPopup,
        },
    },
};

impl AppRenderer {
    pub fn draw(
        screen_manager: &mut ScreenManager,
        state: &RenderState<'_>,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        let overlay = OverlayBus::top_overlay(state);
        let command_buffer = state.command_buffer;
        let render_stack = state.render_stack;
        let key_bindings = state.key_bindings;
        let mode = state.mode;
        let screen_type = state.app_state.current_screen;
        let render_ctx = render_context(state.cfg_state, mode);

        terminal.draw(|frame| {
            draw_screens(
                frame,
                screen_manager,
                render_stack,
                key_bindings,
                mode,
                &render_ctx,
            );
            let command_drawn = matches!(overlay, Some(OverlayItem::CommandLine(_)));
            let allow_command_fallback =
                matches!(overlay, Some(OverlayItem::Notification(_)) | None);
            draw_overlay(frame, overlay, screen_type, key_bindings, &render_ctx);
            draw_command_line_fallback(
                frame,
                allow_command_fallback,
                command_drawn,
                command_buffer,
                &render_ctx,
            );
        })?;

        Ok(())
    }
}

fn render_context(cfg_state: &AppConfigState, mode: crate::app::state::Mode) -> RenderContext {
    match cfg_state {
        AppConfigState::Loaded(cfg) => RenderContext::from_config(cfg, mode),
        AppConfigState::Missing(_) => RenderContext::from_config(&AppConfig::default(), mode),
    }
}

fn draw_screens(
    frame: &mut Frame,
    screen_manager: &mut ScreenManager,
    render_stack: super::RenderStack<'_>,
    key_bindings: &crate::app::key_handlers::KeyBindings,
    mode: crate::app::state::Mode,
    render_ctx: &RenderContext,
) {
    for screen_type in render_stack.iter() {
        if let Some(screen) = screen_manager.screen_mut_existing(screen_type) {
            let hints = action_hints(screen_type, key_bindings);
            screen.set_action_hints(hints);
            screen.set_mode(mode);
            screen.draw(frame, render_ctx);
        }
    }
}

fn draw_overlay(
    frame: &mut Frame,
    overlay: Option<OverlayItem<'_>>,
    screen_type: crate::app::state::ScreenType,
    key_bindings: &crate::app::key_handlers::KeyBindings,
    render_ctx: &RenderContext,
) {
    match overlay {
        Some(OverlayItem::Error(error)) => {
            frame.render_widget(ErrorModal::new(error, render_ctx), frame.area());
        }
        Some(OverlayItem::SyncError(error)) => {
            frame.render_widget(SyncErrorModal::new(error, render_ctx), frame.area());
        }
        Some(OverlayItem::Notification(notifications)) => {
            frame.render_widget(
                NotificationModal::new(notifications, render_ctx),
                frame.area(),
            );
        }
        Some(OverlayItem::CommandLine(buffer)) => {
            frame.render_widget(CommandLineModal::new(buffer, render_ctx), frame.area());
        }
        Some(OverlayItem::WhichKey(mode)) => {
            let popup = match mode {
                WhichKeyMode::Screen => WhichKeyPopup::new(
                    "Key Hints".to_string(),
                    binding_hints_for_screen(screen_type, key_bindings),
                    render_ctx,
                ),
                WhichKeyMode::Prefix(prefix) => WhichKeyPopup::new(
                    format!("Keys: {prefix}"),
                    binding_hints_for_prefix(screen_type, prefix, key_bindings),
                    render_ctx,
                ),
            };
            frame.render_widget(&popup, frame.area());
        }
        Some(OverlayItem::BoardRequired(bindings)) => {
            frame.render_widget(BoardRequiredModal::new(bindings, render_ctx), frame.area());
        }
        None => {}
    }
}

fn draw_command_line_fallback(
    frame: &mut Frame,
    allow_command_fallback: bool,
    command_drawn: bool,
    command_buffer: Option<&str>,
    render_ctx: &RenderContext,
) {
    if !command_drawn
        && allow_command_fallback
        && let Some(buffer) = command_buffer
    {
        frame.render_widget(CommandLineModal::new(buffer, render_ctx), frame.area());
    }
}
