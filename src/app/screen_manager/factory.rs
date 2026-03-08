use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::{
    app::{ProfileEditorIntent, worker_controller::SyncStatusSnapshot},
    data::SyncLogFilter,
    ui::components::logo::AsciiLogoComponent,
};

impl ScreenManager {
    pub(super) async fn create_entry_impl(
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
                let (profiles, active_id) = profiles_view_data(ctx.cfg_state);
                Ok(ScreenEntry::Profiles(ProfilesScreen::new(
                    &profiles, active_id,
                )))
            }
            ScreenType::ProfileCreation => {
                let profile = profile_editor_initial(ctx.cfg_state, ctx.app_state);
                Ok(ScreenEntry::ProfileCreation(ProfileCreationScreen::new(
                    profile,
                )))
            }
            ScreenType::CurrentSprint => {
                let board_id = require_selected_board(ctx.app_state, "Current Sprint")?;
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
                    .sync_log(10, SyncLogFilter::All)
                    .await
                    .unwrap_or_default();
                Ok(ScreenEntry::SyncStatus(SyncStatusScreen::new(
                    SyncStatusSnapshot::default(),
                    log,
                    SyncLogFilter::All,
                )))
            }
            ScreenType::Settings => Ok(ScreenEntry::Settings(SettingsScreen::new())),
            ScreenType::SettingsThemes => {
                let (theme, custom) = theme_view_data(ctx.cfg_state);
                Ok(ScreenEntry::SettingsThemes(SettingsThemesScreen::new(
                    theme, custom,
                )))
            }
            ScreenType::SettingsThemeForm => {
                let (theme, custom) = theme_view_data(ctx.cfg_state);
                Ok(ScreenEntry::SettingsThemeForm(
                    SettingsThemeFormScreen::new(theme, custom),
                ))
            }
            ScreenType::MyIssues => {
                let board_id = require_selected_board(ctx.app_state, "My Issues")?;
                let screen =
                    MyIssuesScreen::new(ctx.repo.clone(), ctx.app_state.mode, board_id).await?;
                Ok(ScreenEntry::MyIssues(screen))
            }
            ScreenType::SearchIssues => {
                let board_id = require_selected_board(ctx.app_state, "Search Issues")?;
                let screen =
                    SearchIssuesScreen::new(ctx.repo.clone(), ctx.app_state.mode, board_id).await?;
                Ok(ScreenEntry::SearchIssues(screen))
            }
            ScreenType::NewIssue => Ok(ScreenEntry::IssueForm(IssueFormScreen::new())),
        }
    }
}

fn profiles_view_data(
    cfg_state: &AppConfigState,
) -> (Vec<crate::config::ProfileConfig>, Option<&str>) {
    match cfg_state {
        AppConfigState::Loaded(cfg) => (cfg.profiles.clone(), cfg.active_profile_id.as_deref()),
        AppConfigState::Missing(_) => (Vec::new(), None),
    }
}

fn profile_editor_initial(
    cfg_state: &AppConfigState,
    app_state: &AppState,
) -> Option<crate::config::ProfileConfig> {
    match app_state.profile_editor.as_ref() {
        Some(ProfileEditorIntent::Edit(id)) => match cfg_state {
            AppConfigState::Loaded(cfg) => cfg
                .profiles
                .iter()
                .find(|profile| &profile.id == id)
                .cloned(),
            AppConfigState::Missing(_) => None,
        },
        Some(ProfileEditorIntent::New) | None => None,
    }
}

fn theme_view_data(cfg_state: &AppConfigState) -> (&str, &[crate::config::CustomThemeConfig]) {
    match cfg_state {
        AppConfigState::Loaded(cfg) => (cfg.ui.theme.as_str(), cfg.ui.custom_themes.as_slice()),
        AppConfigState::Missing(_) => ("default", &[][..]),
    }
}

fn require_selected_board(app_state: &AppState, screen_name: &str) -> Result<u64> {
    app_state
        .selected_board_id
        .ok_or_else(|| eyre!("No board selected: cannot open {}", screen_name))
}
