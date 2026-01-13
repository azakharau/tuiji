use crate::{
    app::key_handlers::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::IssueDetailState;

pub struct IssueDetailController;

impl IssueDetailController {
    pub fn handle_command(state: &mut IssueDetailState, command: Command) -> ScreenState {
        match command.action {
            ActionId::MoveDown => {
                state.scroll_down();
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                state.scroll_up();
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.scroll_to_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.scroll_to_bottom();
                ScreenState::Refresh
            }
            ActionId::PageDown => {
                state.page_down();
                ScreenState::Refresh
            }
            ActionId::PageUp => {
                state.page_up();
                ScreenState::Refresh
            }
            ActionId::OpenInBrowser => ScreenState::OpenInBrowser(state.issue().key.clone()),
            ActionId::Refresh => {
                // Invalidate cache and reload issue
                state.invalidate_cache();
                ScreenState::ViewIssue(state.issue().key.clone())
            }
            ActionId::MoveRight => {
                state.scroll_right();
                ScreenState::Refresh
            }
            ActionId::MoveLeft => {
                state.scroll_left();
                ScreenState::Refresh
            }
            ActionId::MoveLineStart => {
                state.reset_horizontal_scroll();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}
