use crate::app::state::ScreenType;

pub fn is_modal_screen(screen: ScreenType) -> bool {
    matches!(
        screen,
        ScreenType::Profiles
            | ScreenType::ProfileCreation
            | ScreenType::BoardSelection
            | ScreenType::Settings
            | ScreenType::SettingsThemes
            | ScreenType::SettingsThemeForm
    )
}

pub fn has_modal_stack(current_screen: ScreenType, screen_stack: &[ScreenType]) -> bool {
    is_modal_screen(current_screen) || screen_stack.iter().any(|screen| is_modal_screen(*screen))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_modal_stack_from_current_screen() {
        assert!(has_modal_stack(ScreenType::Settings, &[ScreenType::Home]));
    }

    #[test]
    fn detects_modal_stack_from_back_stack() {
        assert!(has_modal_stack(
            ScreenType::CurrentSprint,
            &[ScreenType::Home, ScreenType::Profiles]
        ));
    }

    #[test]
    fn ignores_non_modal_stack() {
        assert!(!has_modal_stack(
            ScreenType::CurrentSprint,
            &[ScreenType::Home, ScreenType::MyIssues]
        ));
    }
}
