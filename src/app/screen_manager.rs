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
    ui::{
        components::logo::AsciiLogoComponent,
        screens::{
            Screen, board_selection::BoardSelectionScreen,
            current_sprint::kanban::CurrentKanbanSprintScreen, home::HomeScreen,
            profile_creation::ProfileCreationScreen, profiles::ProfilesScreen,
        },
    },
};

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
    CurrentSprint(CurrentKanbanSprintScreen),
    Profiles(ProfilesScreen),
    ProfileCreation(ProfileCreationScreen),
}

impl ScreenEntry {
    fn as_screen_mut(&mut self) -> &mut dyn Screen {
        match self {
            ScreenEntry::Home(screen) => screen,
            ScreenEntry::BoardSelection(screen) => screen,
            ScreenEntry::CurrentSprint(screen) => screen,
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

    pub fn default() -> Self {
        Self::new(DEFAULT_SCREEN_TTL)
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
        let slot = self
            .screens
            .get_mut(&screen_type)
            .expect("Screen not available");
        slot.last_used = Instant::now();
        Ok(slot.screen.as_screen_mut())
    }

    pub fn ensure_profiles(&mut self, cfg_state: &AppConfigState) -> &mut ProfilesScreen {
        self.evict_expired();
        if !self.screens.contains_key(&ScreenType::Profiles) {
            let (profiles, active_id) = match cfg_state {
                AppConfigState::Loaded(cfg) => {
                    (cfg.profiles.clone(), cfg.active_profile_id.as_deref())
                }
                AppConfigState::Missing(_) => (Vec::new(), None),
            };
            self.insert(
                ScreenType::Profiles,
                ScreenEntry::Profiles(ProfilesScreen::new(&profiles, active_id)),
            );
        }
        let slot = self
            .screens
            .get_mut(&ScreenType::Profiles)
            .expect("Profiles screen missing");
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::Profiles(screen) => screen,
            _ => panic!("Profiles screen mismatch"),
        }
    }

    pub fn profiles_mut(&mut self) -> Option<&mut ProfilesScreen> {
        let slot = self.screens.get_mut(&ScreenType::Profiles)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::Profiles(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn board_selection_mut(&mut self) -> Option<&mut BoardSelectionScreen> {
        let slot = self.screens.get_mut(&ScreenType::BoardSelection)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::BoardSelection(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn profile_creation_mut(&mut self) -> Option<&mut ProfileCreationScreen> {
        let slot = self.screens.get_mut(&ScreenType::ProfileCreation)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::ProfileCreation(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn screen_mut_existing(&mut self, screen_type: ScreenType) -> Option<&mut dyn Screen> {
        self.evict_expired();
        let slot = self.screens.get_mut(&screen_type)?;
        slot.last_used = Instant::now();
        Some(slot.screen.as_screen_mut())
    }

    pub fn invalidate(&mut self, screen_type: ScreenType) {
        self.screens.remove(&screen_type);
    }

    pub fn invalidate_many(&mut self, screen_types: &[ScreenType]) {
        for screen_type in screen_types {
            self.invalidate(*screen_type);
        }
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
        match screen_type {
            ScreenType::Home => Ok(ScreenEntry::Home(HomeScreen::new(
                AsciiLogoComponent::default(),
                ctx.cfg_state,
            ))),
            ScreenType::BoardSelection => {
                let boards = ctx.repo.list_boards().await.unwrap_or_default();
                Ok(ScreenEntry::BoardSelection(BoardSelectionScreen::new(
                    boards,
                )))
            }
            ScreenType::Profiles => {
                let (profiles, active_id) = match ctx.cfg_state {
                    AppConfigState::Loaded(cfg) => {
                        (cfg.profiles.clone(), cfg.active_profile_id.as_deref())
                    }
                    AppConfigState::Missing(_) => (Vec::new(), None),
                };
                Ok(ScreenEntry::Profiles(ProfilesScreen::new(
                    &profiles, active_id,
                )))
            }
            ScreenType::ProfileCreation => {
                let profile = match ctx.app_state.profile_editor.as_ref() {
                    Some(crate::app::ProfileEditorIntent::Edit(id)) => match ctx.cfg_state {
                        AppConfigState::Loaded(cfg) => {
                            cfg.profiles.iter().find(|p| &p.id == id).cloned()
                        }
                        AppConfigState::Missing(_) => None,
                    },
                    Some(crate::app::ProfileEditorIntent::New) | None => None,
                };
                Ok(ScreenEntry::ProfileCreation(ProfileCreationScreen::new(
                    profile,
                )))
            }
            ScreenType::CurrentSprint => {
                let board_id = ctx.app_state.selected_board_id.ok_or_else(|| {
                    color_eyre::eyre::eyre!("No board selected: cannot open Current Sprint")
                })?;
                let screen =
                    CurrentKanbanSprintScreen::new(ctx.repo.clone(), ctx.app_state.mode, board_id)
                        .await?;
                Ok(ScreenEntry::CurrentSprint(screen))
            }
            _ => Err(color_eyre::eyre::eyre!(
                "Screen {:?} not implemented yet",
                screen_type
            )),
        }
    }
}
