use crate::app::state::ScreenType;

pub(crate) fn command_mode_allowed(screen: ScreenType) -> bool {
    !matches!(screen, ScreenType::Home)
}

pub(crate) fn is_jira_screen(screen: ScreenType) -> bool {
    matches!(
        screen,
        ScreenType::CurrentSprint
            | ScreenType::MyIssues
            | ScreenType::SearchIssues
            | ScreenType::NewIssue
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_mode_is_disabled_on_home() {
        assert!(!command_mode_allowed(ScreenType::Home));
        assert!(command_mode_allowed(ScreenType::CurrentSprint));
    }

    #[test]
    fn jira_screens_match_sync_enabled_views() {
        assert!(is_jira_screen(ScreenType::CurrentSprint));
        assert!(is_jira_screen(ScreenType::MyIssues));
        assert!(is_jira_screen(ScreenType::SearchIssues));
        assert!(is_jira_screen(ScreenType::NewIssue));
        assert!(!is_jira_screen(ScreenType::Home));
        assert!(!is_jira_screen(ScreenType::Profiles));
    }
}
