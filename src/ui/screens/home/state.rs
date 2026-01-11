use crate::{
    config::AppConfigState,
    ui::components::menu::{Menu, MenuItem},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum HomeVariant {
    Welcome,
    #[default]
    Default,
}

pub struct HomeState {
    menu: Menu,
    variant: HomeVariant,
    welcome_text: String,
}

impl HomeState {
    pub fn new(cfg: &AppConfigState, _conflict_count: usize) -> Self {
        let variant = match cfg {
            AppConfigState::Loaded(cfg) => {
                if cfg.profiles.is_empty() {
                    HomeVariant::Welcome
                } else {
                    HomeVariant::Default
                }
            }
            AppConfigState::Missing(_) => HomeVariant::Welcome,
        };
        Self::with_variant(variant)
    }

    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    pub fn menu_mut(&mut self) -> &mut Menu {
        &mut self.menu
    }

    pub fn variant(&self) -> HomeVariant {
        self.variant
    }

    pub fn welcome_text(&self) -> &str {
        self.welcome_text.as_str()
    }

    fn with_variant(variant: HomeVariant) -> Self {
        let menu = Self::menu_for_variant(variant);
        let welcome_text =
            "Welcome! It looks like this is your first time here — let’s create a config to get you set up for work."
                .to_string();
        Self {
            menu,
            variant,
            welcome_text,
        }
    }

    fn menu_for_variant(variant: HomeVariant) -> Menu {
        match variant {
            HomeVariant::Welcome => Menu::new(vec![
                MenuItem::new("ok", "Ok"),
                MenuItem::new("quit", "Quit"),
            ]),
            HomeVariant::Default => Menu::new(vec![
                MenuItem::new("current_sprint", "Current Sprint").with_hint("c"),
                MenuItem::new("my_issues", "My issues").with_hint("i"),
                MenuItem::new("new_issue", "New Issue").with_hint("n"),
                MenuItem::new("boards", "Boards").with_hint("b"),
                MenuItem::new("sync_status", "Sync Status").with_hint("t"),
                MenuItem::new("settings", "Settings").with_hint(","),
                MenuItem::new("quit", "Quit").with_hint("q"),
            ]),
        }
    }
}
