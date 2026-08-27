use std::{collections::HashMap, sync::Arc};

use crate::{
    config::{BindingAction, KeyBindingsConfig},
    ui::interaction::ScreenType,
};

mod hints;
mod mapping;

pub use hints::{
    action_hints, binding_hints_for_prefix, binding_hints_for_screen, is_motion_action,
};
use mapping::{map_bindings, merge_bindings};

pub use crate::ui::interaction::{ActionHint, ActionId, Command, InsertMode, KeyHandler};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    pub action: ActionId,
    pub binding: String,
}

#[derive(Clone, Debug)]
pub struct KeyBindings {
    by_screen: HashMap<ScreenType, Arc<Vec<KeyBinding>>>,
}

impl KeyBindings {
    pub fn from_config(cfg: &KeyBindingsConfig) -> Self {
        let mut global = map_bindings(&cfg.global);
        if !global
            .iter()
            .any(|entry| entry.action == ActionId::OpenSettings)
        {
            global.push(KeyBinding {
                action: ActionId::OpenSettings,
                binding: ",".to_string(),
            });
        }
        let mut by_screen = HashMap::new();
        by_screen.insert(
            ScreenType::Home,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.home))),
        );
        by_screen.insert(
            ScreenType::BoardSelection,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.board_selection))),
        );
        by_screen.insert(
            ScreenType::CurrentSprint,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.current_sprint))),
        );
        by_screen.insert(
            ScreenType::IssueDetail,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.issue_detail))),
        );
        by_screen.insert(
            ScreenType::ProfileCreation,
            Arc::new(merge_bindings(
                &global,
                &map_bindings(&cfg.profile_creation),
            )),
        );
        by_screen.insert(
            ScreenType::Profiles,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.profiles))),
        );
        by_screen.insert(
            ScreenType::MyIssues,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.my_issues))),
        );
        by_screen.insert(
            ScreenType::SearchIssues,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.search_issues))),
        );
        by_screen.insert(
            ScreenType::Conflicts,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.conflicts))),
        );
        by_screen.insert(
            ScreenType::SyncStatus,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.sync_status))),
        );
        by_screen.insert(
            ScreenType::NewIssue,
            Arc::new(merge_bindings(&global, &map_bindings(&cfg.new_issue))),
        );
        let settings_cfg = if cfg.settings.is_empty() {
            KeyBindingsConfig::default().settings
        } else {
            cfg.settings.clone()
        };
        by_screen.insert(
            ScreenType::Settings,
            Arc::new(merge_bindings(&global, &map_bindings(&settings_cfg))),
        );
        by_screen.insert(
            ScreenType::SettingsThemes,
            Arc::new(merge_bindings(&global, &map_bindings(&settings_cfg))),
        );
        by_screen.insert(
            ScreenType::SettingsThemeForm,
            Arc::new(merge_bindings(
                &global,
                &map_bindings(&cfg.profile_creation),
            )),
        );
        Self { by_screen }
    }

    pub fn bindings_for_screen(&self, screen: ScreenType) -> Arc<Vec<KeyBinding>> {
        self.by_screen
            .get(&screen)
            .cloned()
            .unwrap_or_else(|| Arc::new(Vec::new()))
    }

    pub fn bindings_for_screen_ref(&self, screen: ScreenType) -> &[KeyBinding] {
        self.by_screen
            .get(&screen)
            .map(|bindings| bindings.as_slice())
            .unwrap_or(&[])
    }

    pub fn action_for_binding(&self, screen: ScreenType, binding: &str) -> Option<ActionId> {
        self.by_screen.get(&screen).and_then(|bindings| {
            bindings
                .iter()
                .find(|entry| entry.binding == binding)
                .map(|entry| entry.action)
        })
    }

    pub fn binding_strings_for_screen(&self, screen: ScreenType) -> Vec<String> {
        self.by_screen
            .get(&screen)
            .map(|bindings| bindings.iter().map(|b| b.binding.clone()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KeyBindingsConfig;

    #[test]
    fn default_workspace_screens_should_include_vim_navigation_bindings() {
        let bindings = KeyBindings::from_config(&KeyBindingsConfig::default());

        for screen in [ScreenType::MyIssues, ScreenType::SearchIssues] {
            let screen_bindings = bindings.binding_strings_for_screen(screen);
            assert!(screen_bindings.iter().any(|binding| binding == "j"));
            assert!(screen_bindings.iter().any(|binding| binding == "k"));
            assert!(screen_bindings.iter().any(|binding| binding == "gg"));
            assert!(screen_bindings.iter().any(|binding| binding == "G"));
            assert!(screen_bindings.iter().any(|binding| binding == "<enter>"));
        }
    }
}
