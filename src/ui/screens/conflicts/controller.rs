use crate::{
    ui::interaction::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::{ConflictsState, PendingResolve};

pub struct ConflictsController;

impl ConflictsController {
    pub fn handle_command(state: &mut ConflictsState, command: Command) -> ScreenState {
        if let Some(pending) = state.pending_resolve() {
            return match command.action {
                ActionId::Confirm => {
                    state.clear_pending_resolve();
                    match pending {
                        PendingResolve::Local => state
                            .selected_issue_key()
                            .map(|key| ScreenState::ResolveConflictLocal(key.to_string()))
                            .unwrap_or(ScreenState::Stay),
                        PendingResolve::Remote => state
                            .selected_issue_key()
                            .map(|key| ScreenState::ResolveConflictRemote(key.to_string()))
                            .unwrap_or(ScreenState::Stay),
                    }
                }
                ActionId::Quit => {
                    state.clear_pending_resolve();
                    ScreenState::Refresh
                }
                ActionId::ResolveConflictLocal => {
                    state.request_resolve(PendingResolve::Local);
                    ScreenState::Refresh
                }
                ActionId::ResolveConflictRemote => {
                    state.request_resolve(PendingResolve::Remote);
                    ScreenState::Refresh
                }
                _ => ScreenState::Stay,
            };
        }

        match command.action {
            ActionId::MoveUp => {
                state.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                state.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.move_bottom();
                ScreenState::Refresh
            }
            ActionId::ResolveConflictLocal => {
                state.request_resolve(PendingResolve::Local);
                ScreenState::Refresh
            }
            ActionId::ResolveConflictRemote => {
                state.request_resolve(PendingResolve::Remote);
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}
