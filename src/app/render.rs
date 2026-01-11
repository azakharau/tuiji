use std::{collections::VecDeque, sync::Arc};

use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;

use crate::{
    app::{
        AppState,
        error::AppErrorState,
        key_handlers::{
            KeyBindings, action_hints, binding_hints_for_prefix, binding_hints_for_screen,
        },
        notification::AppNotification,
        overlay::{OverlayBus, OverlayItem, WhichKeyMode},
        screen_manager::{ScreenContext, ScreenManager},
        state::{Mode, ScreenType},
        worker_controller::SyncStatusSnapshot,
    },
    config::{AppConfig, AppConfigState},
    data::{AppRepository, RepositoryHub},
    ui::context::RenderContext,
    ui::overlays::{
        BoardRequiredModal, CommandLineModal, ErrorModal, NotificationModal, SyncErrorModal,
        WhichKeyPopup,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct RenderStack<'a> {
    current: ScreenType,
    stack: &'a [ScreenType],
    include_stack: bool,
}

impl<'a> RenderStack<'a> {
    pub fn new(current: ScreenType, stack: &'a [ScreenType], include_stack: bool) -> Self {
        Self {
            current,
            stack,
            include_stack,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ScreenType> + '_ {
        let stack = if self.include_stack { self.stack } else { &[] };
        stack.iter().copied().chain(std::iter::once(self.current))
    }
}

pub struct RenderState<'a> {
    pub cfg_state: &'a AppConfigState,
    pub app_state: &'a AppState,
    pub repo: &'a Arc<RepositoryHub>,
    pub render_stack: RenderStack<'a>,
    pub key_bindings: &'a KeyBindings,
    pub error: Option<&'a AppErrorState>,
    pub notifications: &'a VecDeque<AppNotification>,
    pub command_buffer: Option<&'a str>,
    pub pending_prefix: Option<char>,
    pub show_hints: bool,
    pub board_required: Option<crate::app::overlay::BoardRequiredBindings<'a>>,
    pub mode: Mode,
    pub sync_paused: bool,
    pub sync_error: Option<&'a str>,
    pub sync_status: SyncStatusSnapshot,
}

pub struct AppRenderer;

impl AppRenderer {
    pub async fn prepare(
        screen_manager: &mut ScreenManager,
        state: &RenderState<'_>,
    ) -> Result<()> {
        for screen_type in state.render_stack.iter() {
            let ctx = ScreenContext {
                cfg_state: state.cfg_state,
                app_state: state.app_state,
                repo: state.repo.clone(),
            };
            let _ = screen_manager.active_screen_mut(screen_type, ctx).await?;
        }
        if state
            .render_stack
            .iter()
            .any(|screen| screen == ScreenType::SyncStatus)
        {
            if let Some(screen) = screen_manager.sync_status_mut() {
                screen.set_snapshot(state.sync_status.clone());
                let filter = screen.filter();
                if let Ok(entries) = state.repo.sync_log(10, filter).await {
                    screen.set_log(entries);
                }
            }
        }
        Ok(())
    }

    pub fn draw(
        screen_manager: &mut ScreenManager,
        state: &RenderState<'_>,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        let overlay = OverlayBus::top_overlay(state);
        let command_buffer = state.command_buffer;
        let mut command_drawn = false;
        let render_stack = state.render_stack;
        let key_bindings = state.key_bindings;
        let mode = state.mode;
        let screen_type = state.app_state.current_screen;
        let render_ctx = match state.cfg_state {
            AppConfigState::Loaded(cfg) => RenderContext::from_config(cfg, mode),
            AppConfigState::Missing(_) => RenderContext::from_config(&AppConfig::default(), mode),
        };

        terminal.draw(|frame| {
            for screen_type in render_stack.iter() {
                if let Some(screen) = screen_manager.screen_mut_existing(screen_type) {
                    let hints = action_hints(screen_type, key_bindings);
                    screen.set_action_hints(hints);
                    screen.set_mode(mode);
                    screen.draw(frame, &render_ctx);
                }
            }

            match overlay {
                Some(OverlayItem::Error(error)) => {
                    frame.render_widget(ErrorModal::new(error, &render_ctx), frame.area());
                }
                Some(OverlayItem::SyncError(error)) => {
                    frame.render_widget(SyncErrorModal::new(error, &render_ctx), frame.area());
                }
                Some(OverlayItem::Notification(notifications)) => {
                    frame.render_widget(
                        NotificationModal::new(notifications, &render_ctx),
                        frame.area(),
                    );
                }
                Some(OverlayItem::CommandLine(buffer)) => {
                    frame.render_widget(CommandLineModal::new(buffer, &render_ctx), frame.area());
                    command_drawn = true;
                }
                Some(OverlayItem::WhichKey(mode)) => {
                    let popup = match mode {
                        WhichKeyMode::Screen => WhichKeyPopup::new(
                            "Key Hints".to_string(),
                            binding_hints_for_screen(screen_type, key_bindings),
                            &render_ctx,
                        ),
                        WhichKeyMode::Prefix(prefix) => WhichKeyPopup::new(
                            format!("Keys: {prefix}"),
                            binding_hints_for_prefix(screen_type, prefix, key_bindings),
                            &render_ctx,
                        ),
                    };
                    frame.render_widget(&popup, frame.area());
                }
                Some(OverlayItem::BoardRequired(bindings)) => {
                    frame.render_widget(
                        BoardRequiredModal::new(bindings, &render_ctx),
                        frame.area(),
                    );
                }
                None => {}
            }

            if !command_drawn {
                if let Some(buffer) = command_buffer {
                    if matches!(overlay, Some(OverlayItem::Notification(_)) | None) {
                        frame.render_widget(
                            CommandLineModal::new(buffer, &render_ctx),
                            frame.area(),
                        );
                    }
                }
            }
        })?;

        Ok(())
    }
}
