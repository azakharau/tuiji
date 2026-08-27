use color_eyre::{Result, eyre::eyre};

use super::*;
use crate::{
    app::{FormPurpose, ProfileEditorIntent, worker_controller::SyncStatusSnapshot},
    data::SyncLogFilter,
    ui::components::logo::AsciiLogoComponent,
};

impl ScreenManager {
    pub(super) async fn create_entry_impl(
        &self,
        screen_type: ScreenType,
        ctx: ScreenContext<'_>,
        load_transitions: bool,
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
                let (issues, error) = match ctx.repo.my_issues().await {
                    Ok(issues) => (issues, None),
                    Err(error) => (Vec::new(), Some(error.to_string())),
                };
                Ok(ScreenEntry::MyIssues(MyIssuesScreen::new(
                    ctx.app_state.mode,
                    issues,
                    error,
                )))
            }
            ScreenType::SearchIssues => {
                let query = ctx
                    .app_state
                    .search_issues_query
                    .clone()
                    .unwrap_or_default();
                let (issues, error) = if query.trim().is_empty() {
                    (Vec::new(), None)
                } else {
                    match ctx.repo.search_issues(&query).await {
                        Ok(issues) => (issues, None),
                        Err(error) => (Vec::new(), Some(error.to_string())),
                    }
                };
                Ok(ScreenEntry::SearchIssues(SearchIssuesScreen::new(
                    ctx.app_state.mode,
                    query,
                    issues,
                    error,
                )))
            }
            ScreenType::IssueDetail => {
                let Some(key) = ctx.app_state.issue_detail_key.as_deref() else {
                    return Ok(ScreenEntry::IssueDetail(IssueDetailScreen::unavailable(
                        "No issue was selected".to_string(),
                        ctx.app_state.mode,
                    )));
                };
                let issue = match ctx.repo.issue_by_key(key).await {
                    Ok(Some(issue)) => issue,
                    Ok(Option::None) => {
                        return Ok(ScreenEntry::IssueDetail(IssueDetailScreen::unavailable(
                            format!("{key} is not available in the local cache"),
                            ctx.app_state.mode,
                        )));
                    }
                    Err(error) => {
                        return Ok(ScreenEntry::IssueDetail(IssueDetailScreen::unavailable(
                            error.to_string(),
                            ctx.app_state.mode,
                        )));
                    }
                };
                let transition_result = if load_transitions {
                    Some(
                        ctx.repo
                            .available_transitions(key)
                            .await
                            .map_err(|error| error.to_string()),
                    )
                } else {
                    None
                };
                Ok(ScreenEntry::IssueDetail(IssueDetailScreen::new(
                    issue,
                    ctx.app_state.mode,
                    active_profile_base_url(ctx.cfg_state),
                    transition_result,
                )))
            }
            ScreenType::NewIssue => {
                let purpose = ctx
                    .app_state
                    .issue_form_purpose
                    .clone()
                    .unwrap_or(FormPurpose::Create);
                let (project_key, issue_types, summary, description, load_error) = match &purpose {
                    FormPurpose::Create => {
                        let project_key = selected_project_key(ctx.app_state, &ctx.repo).await;
                        let (issue_types, load_error) = match project_key.as_deref() {
                            Some(project_key) => {
                                match ctx.repo.issue_types(project_key).await {
                                    Ok(issue_types) => (issue_types, Option::None),
                                    Err(err) => (Vec::new(), Some(err.to_string())),
                                }
                            }
                            Option::None => (
                                Vec::new(),
                                Some(
                                    "No board is selected, so the project to create the issue in is unknown"
                                        .to_string(),
                                ),
                            ),
                        };
                        (
                            project_key,
                            issue_types,
                            String::new(),
                            Option::None,
                            load_error,
                        )
                    }
                    FormPurpose::Edit(key) => {
                        let issue = ctx.repo.issue_by_key(key).await?;
                        let summary = issue
                            .as_ref()
                            .map(|issue| issue.summary.clone())
                            .unwrap_or_default();
                        let description = issue.and_then(|issue| issue.description);
                        (Option::None, Vec::new(), summary, description, Option::None)
                    }
                };
                let mut screen =
                    IssueFormScreen::new(purpose, project_key, issue_types, summary, description);
                if let Some(load_error) = load_error {
                    screen.set_error(crate::contracts::error::AppErrorState::error(load_error));
                }
                Ok(ScreenEntry::IssueForm(screen))
            }
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
        Some(ProfileEditorIntent::New) | Option::None => None,
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

fn active_profile_base_url(cfg_state: &AppConfigState) -> Option<String> {
    match cfg_state {
        AppConfigState::Loaded(cfg) => cfg
            .active_profile()
            .map(|profile| profile.jira.base_url.clone()),
        AppConfigState::Missing(_) => None,
    }
}

async fn selected_project_key(
    app_state: &AppState,
    repo: &Arc<dyn AppRepository>,
) -> Option<String> {
    let board_id = app_state.selected_board_id?;
    repo.current_sprint_issues(board_id)
        .await
        .ok()?
        .into_iter()
        .find_map(|issue| issue.project_key)
}
