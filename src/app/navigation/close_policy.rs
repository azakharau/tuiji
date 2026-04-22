use crate::app::state::ScreenType;

use super::{has_modal_stack, is_modal_screen};

pub(crate) fn close_all_modals_target(
    current_screen: ScreenType,
    screen_stack: &[ScreenType],
) -> Option<ScreenType> {
    if !has_modal_stack(current_screen, screen_stack) {
        return None;
    }

    Some(
        screen_stack
            .iter()
            .rev()
            .find(|screen| !is_modal_screen(**screen))
            .copied()
            .unwrap_or(ScreenType::Home),
    )
}

pub(crate) fn should_cleanup_profile_creation(
    current_screen: ScreenType,
    screen_stack: &[ScreenType],
) -> bool {
    current_screen == ScreenType::ProfileCreation
        || screen_stack.contains(&ScreenType::ProfileCreation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_all_modals_returns_none_without_modal_stack() {
        assert_eq!(
            close_all_modals_target(ScreenType::CurrentSprint, &[ScreenType::Home]),
            None
        );
    }

    #[test]
    fn close_all_modals_targets_latest_non_modal_screen() {
        assert_eq!(
            close_all_modals_target(
                ScreenType::SettingsThemeForm,
                &[
                    ScreenType::Home,
                    ScreenType::CurrentSprint,
                    ScreenType::Settings,
                    ScreenType::Profiles,
                ],
            ),
            Some(ScreenType::CurrentSprint)
        );
    }

    #[test]
    fn close_all_modals_falls_back_to_home_for_modal_only_stack() {
        assert_eq!(
            close_all_modals_target(
                ScreenType::Profiles,
                &[ScreenType::Settings, ScreenType::BoardSelection],
            ),
            Some(ScreenType::Home)
        );
    }

    #[test]
    fn cleanup_profile_creation_when_current_screen_is_profile_creation() {
        assert!(should_cleanup_profile_creation(
            ScreenType::ProfileCreation,
            &[ScreenType::Home]
        ));
    }

    #[test]
    fn cleanup_profile_creation_when_present_in_modal_stack() {
        assert!(should_cleanup_profile_creation(
            ScreenType::Settings,
            &[ScreenType::Home, ScreenType::ProfileCreation]
        ));
    }

    #[test]
    fn do_not_cleanup_profile_creation_when_not_present() {
        assert!(!should_cleanup_profile_creation(
            ScreenType::Settings,
            &[ScreenType::Home, ScreenType::Profiles]
        ));
    }
}
