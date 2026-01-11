use crate::{
    app::{
        key_handlers::{ActionId, Command},
        state::ScreenType,
    },
    ui::screens::ScreenState,
};

use super::state::{HomeState, HomeVariant};

pub struct HomeController;

impl HomeController {
    pub fn handle_command(state: &mut HomeState, command: Command) -> ScreenState {
        match command.action {
            ActionId::Refresh => ScreenState::Refresh,
            ActionId::MoveUp => {
                state.menu_mut().move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                state.menu_mut().move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.menu_mut().move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.menu_mut().move_bottom();
                ScreenState::Refresh
            }
            ActionId::Confirm => Self::handle_confirm(state),
            _ => ScreenState::Stay,
        }
    }

    fn handle_confirm(state: &HomeState) -> ScreenState {
        let Some(item) = state.menu().selected() else {
            return ScreenState::Stay;
        };

        match (state.variant(), item.id) {
            (HomeVariant::Welcome, "ok") => ScreenState::SwitchTo(ScreenType::ProfileCreation),
            (HomeVariant::Welcome, "quit") => ScreenState::Quit,
            (HomeVariant::Default, "current_sprint") => {
                ScreenState::SwitchTo(ScreenType::CurrentSprint)
            }
            (HomeVariant::Default, "my_issues") => ScreenState::SwitchTo(ScreenType::MyIssues),
            (HomeVariant::Default, "new_issue") => ScreenState::SwitchTo(ScreenType::NewIssue),
            (HomeVariant::Default, "boards") => ScreenState::SwitchTo(ScreenType::BoardSelection),
            (HomeVariant::Default, "sync_status") => ScreenState::SwitchTo(ScreenType::SyncStatus),
            (HomeVariant::Default, "settings") => ScreenState::SwitchTo(ScreenType::Settings),
            (HomeVariant::Default, "quit") => ScreenState::Quit,
            _ => ScreenState::Stay,
        }
    }
}
