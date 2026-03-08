mod controller;
mod state;
mod view;

use std::sync::Arc;

use ratatui::Frame;

use crate::{
    contracts::sync::SyncStatusSnapshot,
    data::{SyncLogEntry, SyncLogFilter},
    ui::{
        context::RenderContext,
        interaction::{ActionHint, Command, KeyHandler, Mode},
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

use controller::SyncStatusController;
use state::SyncStatusState;
use view::SyncStatusView;

pub struct SyncStatusScreen {
    state: SyncStatusState,
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
}

impl SyncStatusScreen {
    pub fn new(
        snapshot: SyncStatusSnapshot,
        sync_log: Vec<SyncLogEntry>,
        filter: SyncLogFilter,
    ) -> Self {
        Self {
            state: SyncStatusState::new(snapshot, sync_log, filter),
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
        }
    }

    pub fn set_snapshot(&mut self, snapshot: SyncStatusSnapshot) {
        self.state.set_snapshot(snapshot);
    }

    pub fn set_log(&mut self, entries: Vec<SyncLogEntry>) {
        self.state.set_sync_log(entries);
    }

    pub fn filter(&self) -> SyncLogFilter {
        self.state.filter()
    }
}

impl Screen for SyncStatusScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        SyncStatusView::draw(frame, &self.state, self.mode, &self.actions, context);
    }

    fn name(&self) -> &'static str {
        "Sync Status Screen"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => ScreenState::Stay,
            CommandLineCommand::WriteQuit => ScreenState::Close,
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

impl KeyHandler for SyncStatusScreen {
    fn handle_command(&mut self, command: Command) -> crate::ui::screens::ScreenState {
        SyncStatusController::handle_command(&mut self.state, command)
    }
}
