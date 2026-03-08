use std::{collections::VecDeque, sync::Arc};

use color_eyre::eyre::Result;

use crate::{
    app::{
        App, AppState,
        error::AppErrorState,
        key_handlers::KeyBindings,
        notification::AppNotification,
        state::{Mode, ScreenType},
        worker_controller::SyncStatusSnapshot,
    },
    config::AppConfigState,
    data::RepositoryHub,
    ui::interaction::BoardRequiredBindings,
};

mod draw;
mod prepare;

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
    pub board_required: Option<BoardRequiredBindings<'a>>,
    pub mode: Mode,
    pub sync_paused: bool,
    pub sync_error: Option<&'a str>,
    pub sync_status: SyncStatusSnapshot,
}

pub struct AppRenderer;

impl App {
    pub(super) async fn render(&mut self) -> Result<()> {
        let render_stack = crate::app::navigation::build_render_stack(
            self.state.current_screen,
            &self.screen_stack,
        );
        let board_required = if crate::app::navigation::board_required_active(&self.state) {
            Some(crate::app::navigation::board_required_bindings(
                self.state.current_screen,
                self.key_bindings.as_ref(),
            ))
        } else {
            None
        };
        let repo = self.repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot render screens")
        })?;
        let state = RenderState {
            cfg_state: &self.cfg_state,
            app_state: &self.state,
            repo,
            render_stack,
            key_bindings: self.key_bindings.as_ref(),
            error: self.notification_service.error_state(),
            notifications: self.notification_service.items(),
            command_buffer: self.command_line.buffer(),
            pending_prefix: self.input.pending_prefix(),
            show_hints: self.show_hints,
            board_required,
            mode: self.state.mode,
            sync_paused: self.worker_controller.is_paused(),
            sync_error: self.worker_controller.last_error(),
            sync_status: self.worker_controller.snapshot(),
        };
        AppRenderer::prepare(&mut self.screen_manager, &state).await?;
        AppRenderer::draw(&mut self.screen_manager, &state, &mut self.terminal)?;
        Ok(())
    }
}
