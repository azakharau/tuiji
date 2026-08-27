use crate::{
    app::{
        AppState,
        key_handlers::{ActionId, KeyBinding, KeyBindings},
        state::ScreenType,
    },
    ui::interaction::BoardRequiredBindings,
};

pub fn is_board_required_screen(screen: ScreenType) -> bool {
    screen == ScreenType::Home
        || screen == ScreenType::CurrentSprint
        || screen == ScreenType::IssueDetail
        || screen == ScreenType::MyIssues
        || screen == ScreenType::SearchIssues
}

pub fn board_required_active(state: &AppState) -> bool {
    state.selected_board_id.is_none() && is_board_required_screen(state.current_screen)
}

pub fn board_required_bindings<'a>(
    current_screen: ScreenType,
    key_bindings: &'a KeyBindings,
) -> BoardRequiredBindings<'a> {
    board_required_bindings_for_screen(
        current_screen,
        key_bindings.bindings_for_screen_ref(current_screen),
    )
}

fn board_required_bindings_for_screen(
    current_screen: ScreenType,
    bindings: &[KeyBinding],
) -> BoardRequiredBindings<'_> {
    let open_key = bindings
        .iter()
        .find(|entry| entry.action == ActionId::OpenBoards)
        .map(|entry| entry.binding.as_str())
        .unwrap_or("b");
    let profiles_key = bindings
        .iter()
        .find(|entry| entry.action == ActionId::OpenProfiles)
        .map(|entry| entry.binding.as_str());
    let quit_key = bindings
        .iter()
        .find(|entry| entry.action == ActionId::Quit)
        .map(|entry| entry.binding.as_str());

    BoardRequiredBindings {
        open: open_key,
        profiles: profiles_key,
        quit: quit_binding(current_screen, quit_key),
    }
}

fn quit_binding(current_screen: ScreenType, quit_key: Option<&str>) -> Option<&str> {
    (current_screen == ScreenType::Home).then_some(quit_key.unwrap_or("q"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;

    #[test]
    fn board_required_only_when_no_board_selected_for_required_screen() {
        let mut state = AppState {
            current_screen: ScreenType::CurrentSprint,
            ..AppState::default()
        };

        assert!(board_required_active(&state));

        state.selected_board_id = Some(42);
        assert!(!board_required_active(&state));

        state.selected_board_id = None;
        state.current_screen = ScreenType::Settings;
        assert!(!board_required_active(&state));
    }

    #[test]
    fn board_required_bindings_fall_back_to_defaults() {
        let bindings = board_required_bindings_for_screen(ScreenType::Home, &[]);

        assert_eq!(bindings.open, "b");
        assert_eq!(bindings.profiles, None);
        assert_eq!(bindings.quit, Some("q"));
    }

    #[test]
    fn board_required_bindings_use_matching_actions() {
        let bindings = vec![
            KeyBinding {
                action: ActionId::Quit,
                binding: "zz".to_string(),
            },
            KeyBinding {
                action: ActionId::OpenProfiles,
                binding: "gp".to_string(),
            },
            KeyBinding {
                action: ActionId::OpenBoards,
                binding: "gb".to_string(),
            },
        ];

        let resolved = board_required_bindings_for_screen(ScreenType::Home, &bindings);

        assert_eq!(resolved.open, "gb");
        assert_eq!(resolved.profiles, Some("gp"));
        assert_eq!(resolved.quit, Some("zz"));
    }

    #[test]
    fn board_required_bindings_hide_quit_off_home() {
        let resolved = board_required_bindings_for_screen(ScreenType::CurrentSprint, &[]);

        assert_eq!(resolved.quit, None);
    }
}
