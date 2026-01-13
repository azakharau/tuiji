use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use color_eyre::Result;

use crate::{
    app::{
        AppState,
        state::{Mode, ScreenType},
    },
    config::AppConfigState,
    data::AppRepository,
    ui::{
        components::logo::AsciiLogoComponent,
        screens::{
            Screen,
            board_selection::BoardSelectionScreen,
            conflicts::ConflictsScreen,
            current_sprint::CurrentSprintScreen,
            home::HomeScreen,
            issue_detail::IssueDetailScreen,
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
    Conflicts(ConflictsScreen),
    SyncStatus(SyncStatusScreen),
    CurrentSprint(CurrentSprintScreen),
    MyIssues(MyIssuesScreen),
    SearchIssues(SearchIssuesScreen),
    IssueForm(IssueFormScreen),
    IssueDetail(Box<IssueDetailScreen>),
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
            ScreenEntry::IssueDetail(screen) => screen.as_mut(),
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

    pub fn with_default_ttl() -> Self {
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
        // SAFETY: We just inserted the screen if it wasn't present
        let slot = self.screens.get_mut(&screen_type).ok_or_else(|| {
            color_eyre::eyre::eyre!("Screen {:?} not available after creation", screen_type)
        })?;
        slot.last_used = Instant::now();
        Ok(slot.screen.as_screen_mut())
    }

    pub fn ensure_profiles(&mut self, cfg_state: &AppConfigState) -> &mut ProfilesScreen {
        self.evict_expired();
        let slot = self
            .screens
            .entry(ScreenType::Profiles)
            .or_insert_with(|| {
                let (profiles, active_id) = match cfg_state {
                    AppConfigState::Loaded(cfg) => {
                        (cfg.profiles.clone(), cfg.active_profile_id.as_deref())
                    }
                    AppConfigState::Missing(_) => (Vec::new(), None),
                };
                ScreenSlot {
                    screen: ScreenEntry::Profiles(ProfilesScreen::new(&profiles, active_id)),
                    last_used: Instant::now(),
                }
            });
        slot.last_used = Instant::now();
        let ScreenEntry::Profiles(screen) = &mut slot.screen else {
            unreachable!("ScreenType::Profiles always maps to ScreenEntry::Profiles")
        };
        screen
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

    pub fn conflicts_mut(&mut self) -> Option<&mut ConflictsScreen> {
        let slot = self.screens.get_mut(&ScreenType::Conflicts)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::Conflicts(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn sync_status_mut(&mut self) -> Option<&mut SyncStatusScreen> {
        let slot = self.screens.get_mut(&ScreenType::SyncStatus)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::SyncStatus(screen) => Some(screen),
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

    pub fn settings_mut(&mut self) -> Option<&mut SettingsScreen> {
        let slot = self.screens.get_mut(&ScreenType::Settings)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::Settings(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn settings_themes_mut(&mut self) -> Option<&mut SettingsThemesScreen> {
        let slot = self.screens.get_mut(&ScreenType::SettingsThemes)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::SettingsThemes(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn settings_theme_form_mut(&mut self) -> Option<&mut SettingsThemeFormScreen> {
        let slot = self.screens.get_mut(&ScreenType::SettingsThemeForm)?;
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::SettingsThemeForm(screen) => Some(screen),
            _ => None,
        }
    }

    pub fn ensure_settings(&mut self, _cfg_state: &AppConfigState) -> &mut SettingsScreen {
        self.evict_expired();
        let slot = self
            .screens
            .entry(ScreenType::Settings)
            .or_insert_with(|| ScreenSlot {
                screen: ScreenEntry::Settings(SettingsScreen::new()),
                last_used: Instant::now(),
            });
        slot.last_used = Instant::now();
        let ScreenEntry::Settings(screen) = &mut slot.screen else {
            unreachable!("ScreenType::Settings always maps to ScreenEntry::Settings")
        };
        screen
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

    /// Create an IssueDetail screen with the given issue data.
    /// This screen is not cached and must be created on-demand.
    pub fn create_issue_detail(
        &mut self,
        issue: crate::data::IssueSummary,
        mode: Mode,
    ) -> &mut IssueDetailScreen {
        let screen = IssueDetailScreen::new(issue, mode);
        self.screens.insert(
            ScreenType::IssueDetail,
            ScreenSlot {
                screen: ScreenEntry::IssueDetail(Box::new(screen)),
                last_used: Instant::now(),
            },
        );
        // SAFETY: We just inserted this entry
        let slot = self
            .screens
            .get_mut(&ScreenType::IssueDetail)
            .unwrap_or_else(|| unreachable!("IssueDetail was just inserted"));
        let ScreenEntry::IssueDetail(ref mut screen) = slot.screen else {
            unreachable!("ScreenType::IssueDetail always maps to ScreenEntry::IssueDetail")
        };
        screen.as_mut()
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
            ScreenType::Home => {
                let conflict_count = ctx.repo.conflict_count().await.unwrap_or(0);
                Ok(ScreenEntry::Home(HomeScreen::new(
                    AsciiLogoComponent::default(),
                    ctx.cfg_state,
                    conflict_count,
                )))
            }
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
                    CurrentSprintScreen::new(ctx.repo.clone(), ctx.app_state.mode, board_id)
                        .await?;
                Ok(ScreenEntry::CurrentSprint(screen))
            }
            ScreenType::Conflicts => {
                let issues = ctx.repo.conflict_issues().await.unwrap_or_default();
                Ok(ScreenEntry::Conflicts(ConflictsScreen::new(issues)))
            }
            ScreenType::SyncStatus => {
                let log = ctx
                    .repo
                    .sync_log(10, crate::data::SyncLogFilter::All)
                    .await
                    .unwrap_or_default();
                Ok(ScreenEntry::SyncStatus(SyncStatusScreen::new(
                    crate::app::worker_controller::SyncStatusSnapshot::default(),
                    log,
                    crate::data::SyncLogFilter::All,
                )))
            }
            ScreenType::Settings => Ok(ScreenEntry::Settings(SettingsScreen::new())),
            ScreenType::SettingsThemes => {
                let (theme, custom) = match ctx.cfg_state {
                    AppConfigState::Loaded(cfg) => {
                        (cfg.ui.theme.as_str(), cfg.ui.custom_themes.as_slice())
                    }
                    AppConfigState::Missing(_) => ("default", &[][..]),
                };
                Ok(ScreenEntry::SettingsThemes(SettingsThemesScreen::new(
                    theme, custom,
                )))
            }
            ScreenType::SettingsThemeForm => {
                let (theme, custom) = match ctx.cfg_state {
                    AppConfigState::Loaded(cfg) => {
                        (cfg.ui.theme.as_str(), cfg.ui.custom_themes.as_slice())
                    }
                    AppConfigState::Missing(_) => ("default", &[][..]),
                };
                Ok(ScreenEntry::SettingsThemeForm(
                    SettingsThemeFormScreen::new(theme, custom),
                ))
            }
            ScreenType::MyIssues => {
                let board_id = ctx.app_state.selected_board_id.ok_or_else(|| {
                    color_eyre::eyre::eyre!("No board selected: cannot open My Issues")
                })?;
                let screen =
                    MyIssuesScreen::new(ctx.repo.clone(), ctx.app_state.mode, board_id).await?;
                Ok(ScreenEntry::MyIssues(screen))
            }
            ScreenType::SearchIssues => {
                let board_id = ctx.app_state.selected_board_id.ok_or_else(|| {
                    color_eyre::eyre::eyre!("No board selected: cannot open Search Issues")
                })?;
                let screen =
                    SearchIssuesScreen::new(ctx.repo.clone(), ctx.app_state.mode, board_id).await?;
                Ok(ScreenEntry::SearchIssues(screen))
            }
            ScreenType::NewIssue => Ok(ScreenEntry::IssueForm(IssueFormScreen::new())),
            ScreenType::IssueDetail => Err(color_eyre::eyre::eyre!(
                "IssueDetail screen must be created with push_issue_detail() - it requires issue data"
            )),
        }
    }
}
