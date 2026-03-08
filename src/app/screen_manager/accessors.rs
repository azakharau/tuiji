use super::*;

impl ScreenManager {
    pub fn ensure_profiles(&mut self, cfg_state: &AppConfigState) -> Result<&mut ProfilesScreen> {
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
        let Some(slot) = self.screens.get_mut(&ScreenType::Profiles) else {
            return Err(color_eyre::eyre::eyre!("Profiles screen missing"));
        };
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::Profiles(screen) => Ok(screen),
            _ => Err(color_eyre::eyre::eyre!("Profiles screen mismatch")),
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

    pub fn ensure_settings(&mut self, _cfg_state: &AppConfigState) -> Result<&mut SettingsScreen> {
        self.evict_expired();
        if !self.screens.contains_key(&ScreenType::Settings) {
            self.insert(
                ScreenType::Settings,
                ScreenEntry::Settings(SettingsScreen::new()),
            );
        }
        let Some(slot) = self.screens.get_mut(&ScreenType::Settings) else {
            return Err(color_eyre::eyre::eyre!("Settings screen missing"));
        };
        slot.last_used = Instant::now();
        match &mut slot.screen {
            ScreenEntry::Settings(screen) => Ok(screen),
            _ => Err(color_eyre::eyre::eyre!("Settings screen mismatch")),
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
}
