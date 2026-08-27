use crate::{
    app::{
        FormPurpose, screen_policy,
        state::{Mode, ScreenType},
    },
    ui::{interaction::ActionId, screens::ScreenState},
};

pub enum ModeSwitchDecision {
    Apply(Mode),
    CloseModal,
    Reject,
}

pub struct ScreenTransitionPolicy;

impl ScreenTransitionPolicy {
    pub fn mode_switch(
        current_mode: Mode,
        target_mode: Mode,
        current_screen: ScreenType,
        modal_screen: bool,
    ) -> ModeSwitchDecision {
        if target_mode == Mode::Normal && current_mode == Mode::Normal && modal_screen {
            return ModeSwitchDecision::CloseModal;
        }
        if target_mode == Mode::Command && !Self::command_mode_allowed(current_screen) {
            return ModeSwitchDecision::Reject;
        }
        ModeSwitchDecision::Apply(target_mode)
    }

    pub fn board_required_action(current_screen: ScreenType, action: ActionId) -> ScreenState {
        match action {
            ActionId::OpenBoards => ScreenState::SwitchTo(ScreenType::BoardSelection),
            ActionId::OpenProfiles => ScreenState::SwitchTo(ScreenType::Profiles),
            ActionId::Quit if current_screen == ScreenType::Home => ScreenState::Quit,
            ActionId::GoHome => ScreenState::SwitchTo(ScreenType::Home),
            _ => ScreenState::Stay,
        }
    }

    pub fn global_action(current_screen: ScreenType, action: ActionId) -> Option<ScreenState> {
        match action {
            ActionId::Quit => {
                if current_screen == ScreenType::Home {
                    Some(ScreenState::Quit)
                } else {
                    None
                }
            }
            ActionId::Refresh => Some(ScreenState::Refresh),
            ActionId::GoHome => Some(ScreenState::SwitchTo(ScreenType::Home)),
            ActionId::OpenCurrentSprint => Some(ScreenState::SwitchTo(ScreenType::CurrentSprint)),
            ActionId::OpenProfiles => {
                if current_screen == ScreenType::Settings {
                    Some(ScreenState::SwitchTo(ScreenType::Profiles))
                } else {
                    None
                }
            }
            ActionId::OpenMyIssues => Some(ScreenState::SwitchTo(ScreenType::MyIssues)),
            ActionId::OpenSearchIssues => Some(ScreenState::SwitchTo(ScreenType::SearchIssues)),
            ActionId::OpenNewIssue => Some(ScreenState::OpenIssueForm(FormPurpose::Create)),
            ActionId::OpenBoards => Some(ScreenState::SwitchTo(ScreenType::BoardSelection)),
            ActionId::OpenSettings => Some(ScreenState::SwitchTo(ScreenType::Settings)),
            ActionId::OpenSyncStatus => Some(ScreenState::SwitchTo(ScreenType::SyncStatus)),
            _ => None,
        }
    }

    pub fn command_mode_allowed(screen: ScreenType) -> bool {
        screen_policy::command_mode_allowed(screen)
    }

    pub fn is_jira_screen(screen: ScreenType) -> bool {
        screen_policy::is_jira_screen(screen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_switch_should_close_modal_on_double_normal_escape() {
        let decision = ScreenTransitionPolicy::mode_switch(
            Mode::Normal,
            Mode::Normal,
            ScreenType::Profiles,
            true,
        );
        assert!(matches!(decision, ModeSwitchDecision::CloseModal));
    }

    #[test]
    fn command_mode_should_be_rejected_on_home() {
        let decision = ScreenTransitionPolicy::mode_switch(
            Mode::Normal,
            Mode::Command,
            ScreenType::Home,
            false,
        );
        assert!(matches!(decision, ModeSwitchDecision::Reject));
    }

    #[test]
    fn open_profiles_is_scoped_to_settings_screen() {
        let settings =
            ScreenTransitionPolicy::global_action(ScreenType::Settings, ActionId::OpenProfiles);
        let home = ScreenTransitionPolicy::global_action(ScreenType::Home, ActionId::OpenProfiles);
        assert_eq!(settings, Some(ScreenState::SwitchTo(ScreenType::Profiles)));
        assert_eq!(home, None);
    }
}
