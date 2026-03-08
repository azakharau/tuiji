use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use color_eyre::Result;

use crate::{
    app::{AppState, state::ScreenType},
    config::AppConfigState,
    data::AppRepository,
    ui::screens::{
        Screen,
        board_selection::BoardSelectionScreen,
        conflicts::ConflictsScreen,
        current_sprint::CurrentSprintScreen,
        home::HomeScreen,
        issue_form::IssueFormScreen,
        my_issues::MyIssuesScreen,
        profile_creation::ProfileCreationScreen,
        profiles::ProfilesScreen,
        search_issues::SearchIssuesScreen,
        settings::{
            SettingsScreen, theme_form::SettingsThemeFormScreen, themes::SettingsThemesScreen,
        },
        sync_status::SyncStatusScreen,
    },
};

mod accessors;
mod factory;

const DEFAULT_SCREEN_TTL: Duration = Duration::from_secs(60);

pub struct ScreenContext<'a> {
    pub cfg_state: &'a AppConfigState,
    pub app_state: &'a AppState,
    pub repo: Arc<dyn AppRepository>,
}

pub struct ScreenManager {
    ttl: Duration,
    screens: HashMap<ScreenType, ScreenSlot>,
}

struct ScreenSlot {
    screen: ScreenEntry,
    last_used: Instant,
}

enum ScreenEntry {
    Home(HomeScreen),
    BoardSelection(BoardSelectionScreen),
    Conflicts(ConflictsScreen),
    SyncStatus(SyncStatusScreen),
    CurrentSprint(CurrentSprintScreen),
    MyIssues(MyIssuesScreen),
    SearchIssues(SearchIssuesScreen),
    IssueForm(IssueFormScreen),
    Settings(SettingsScreen),
    SettingsThemes(SettingsThemesScreen),
    SettingsThemeForm(SettingsThemeFormScreen),
    Profiles(ProfilesScreen),
    ProfileCreation(ProfileCreationScreen),
}

impl ScreenEntry {
    fn as_screen_mut(&mut self) -> &mut dyn Screen {
        match self {
            ScreenEntry::Home(screen) => screen,
            ScreenEntry::BoardSelection(screen) => screen,
            ScreenEntry::Conflicts(screen) => screen,
            ScreenEntry::SyncStatus(screen) => screen,
            ScreenEntry::CurrentSprint(screen) => screen,
            ScreenEntry::MyIssues(screen) => screen,
            ScreenEntry::SearchIssues(screen) => screen,
            ScreenEntry::IssueForm(screen) => screen,
            ScreenEntry::Settings(screen) => screen,
            ScreenEntry::SettingsThemes(screen) => screen,
            ScreenEntry::SettingsThemeForm(screen) => screen,
            ScreenEntry::Profiles(screen) => screen,
            ScreenEntry::ProfileCreation(screen) => screen,
        }
    }
}

impl ScreenManager {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            screens: HashMap::new(),
        }
    }

    pub async fn active_screen_mut(
        &mut self,
        screen_type: ScreenType,
        ctx: ScreenContext<'_>,
    ) -> Result<&mut dyn Screen> {
        self.evict_expired();
        if !self.screens.contains_key(&screen_type) {
            let entry = self.create_entry(screen_type, ctx).await?;
            self.insert(screen_type, entry);
        }
        let Some(slot) = self.screens.get_mut(&screen_type) else {
            return Err(color_eyre::eyre::eyre!(
                "Screen not available after initialization: {:?}",
                screen_type
            ));
        };
        slot.last_used = Instant::now();
        Ok(slot.screen.as_screen_mut())
    }

    fn insert(&mut self, screen_type: ScreenType, entry: ScreenEntry) {
        self.screens.insert(
            screen_type,
            ScreenSlot {
                screen: entry,
                last_used: Instant::now(),
            },
        );
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        self.screens
            .retain(|_, slot| now.duration_since(slot.last_used) <= self.ttl);
    }

    async fn create_entry(
        &self,
        screen_type: ScreenType,
        ctx: ScreenContext<'_>,
    ) -> Result<ScreenEntry> {
        self.create_entry_impl(screen_type, ctx).await
    }
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self::new(DEFAULT_SCREEN_TTL)
    }
}
