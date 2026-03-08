use std::sync::Arc;

use super::*;

mod descriptions;
mod screen_hints;

use descriptions::action_description;
use screen_hints::screen_hint_actions;

/// Generates bottom-bar hints from the current bindings.
pub fn action_hints(screen: ScreenType, bindings: &KeyBindings) -> Arc<Vec<ActionHint>> {
    let mut hints = Vec::new();
    let bindings = bindings.bindings_for_screen(screen);
    let first = |id: ActionId| {
        bindings
            .iter()
            .find(|entry| entry.action == id)
            .map(|entry| entry.binding.clone())
    };

    let mut push = |id: ActionId, description: &str| {
        if let Some(b) = first(id) {
            hints.push(ActionHint {
                binding: b,
                description: description.to_string(),
            });
        }
    };

    push(ActionId::Refresh, "Refresh");
    for (action, description) in screen_hint_actions(screen) {
        push(*action, description);
    }

    Arc::new(hints)
}

pub fn binding_hints_for_prefix(
    screen: ScreenType,
    prefix: char,
    bindings: &KeyBindings,
) -> Vec<ActionHint> {
    let mut hints = Vec::new();
    for entry in bindings.bindings_for_screen(screen).iter() {
        if !entry.binding.starts_with(prefix) || entry.binding.chars().nth(1).is_none() {
            continue;
        }
        if let Some(description) = action_description(entry.action) {
            hints.push(ActionHint {
                binding: entry.binding.clone(),
                description: description.to_string(),
            });
        }
    }
    hints.sort_by(|a, b| a.binding.cmp(&b.binding));
    hints
}

pub fn binding_hints_for_screen(screen: ScreenType, bindings: &KeyBindings) -> Vec<ActionHint> {
    let mut hints = Vec::new();
    for entry in bindings.bindings_for_screen(screen).iter() {
        if let Some(description) = action_description(entry.action) {
            hints.push(ActionHint {
                binding: entry.binding.clone(),
                description: description.to_string(),
            });
        }
    }
    hints.sort_by(|a, b| a.binding.cmp(&b.binding));
    hints
}

pub fn is_motion_action(action: ActionId) -> bool {
    matches!(
        action,
        ActionId::MoveUp
            | ActionId::MoveDown
            | ActionId::MoveLeft
            | ActionId::MoveRight
            | ActionId::MoveTop
            | ActionId::MoveBottom
            | ActionId::MoveLineStart
            | ActionId::MoveLineEnd
            | ActionId::MoveWordForward
            | ActionId::MoveWordBackward
            | ActionId::MoveWordEnd
    )
}

#[cfg(test)]
mod tests {
    use super::{binding_hints_for_screen, is_motion_action};
    use crate::{
        app::key_handlers::{ActionId, KeyBindings},
        config::KeyBindingsConfig,
        ui::interaction::ScreenType,
    };

    #[test]
    fn binding_hints_for_screen_should_return_sorted_descriptions() {
        let bindings = KeyBindings::from_config(&KeyBindingsConfig::default());

        let hints = binding_hints_for_screen(ScreenType::SyncStatus, &bindings);

        assert!(!hints.is_empty());
        assert!(
            hints
                .windows(2)
                .all(|pair| pair[0].binding <= pair[1].binding)
        );
    }

    #[test]
    fn is_motion_action_should_only_match_navigation_actions() {
        assert!(is_motion_action(ActionId::MoveDown));
        assert!(is_motion_action(ActionId::MoveWordBackward));
        assert!(!is_motion_action(ActionId::Refresh));
        assert!(!is_motion_action(ActionId::OpenSettings));
    }
}
