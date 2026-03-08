use std::sync::Arc;

use color_eyre::Result;

use crate::{
    app::{
        AppState,
        key_handlers::Command,
        screen_manager::{ScreenContext, ScreenManager},
        state::ScreenType,
    },
    config::AppConfigState,
    data::{BoardRepository, RepositoryHub},
    ui::{interaction::ActionId, screens::ScreenState},
};

pub async fn handle_board_selection_command(
    cmd: Command,
    state: &mut AppState,
    screen_stack: &mut Vec<ScreenType>,
    screen_manager: &mut ScreenManager,
    cfg_state: &AppConfigState,
    repo: &Option<Arc<RepositoryHub>>,
) -> Result<ScreenState> {
    match cmd.action {
        ActionId::MoveUp => {
            let screen =
                ensure_board_selection_screen(screen_manager, cfg_state, state, repo).await?;
            screen.move_up(cmd.repeat);
            Ok(ScreenState::Refresh)
        }
        ActionId::MoveDown => {
            let screen =
                ensure_board_selection_screen(screen_manager, cfg_state, state, repo).await?;
            screen.move_down(cmd.repeat);
            Ok(ScreenState::Refresh)
        }
        ActionId::MoveTop => {
            let screen =
                ensure_board_selection_screen(screen_manager, cfg_state, state, repo).await?;
            screen.move_top();
            Ok(ScreenState::Refresh)
        }
        ActionId::MoveBottom => {
            let screen =
                ensure_board_selection_screen(screen_manager, cfg_state, state, repo).await?;
            screen.move_bottom();
            Ok(ScreenState::Refresh)
        }
        ActionId::Confirm => {
            let selected_board_id = {
                let screen =
                    ensure_board_selection_screen(screen_manager, cfg_state, state, repo).await?;
                screen.selected_board_id()
            };
            let Some(board_id) = selected_board_id else {
                return Ok(ScreenState::Stay);
            };
            let repo = repo.as_ref().ok_or_else(|| {
                color_eyre::eyre::eyre!("Repository not initialized: cannot select board")
            })?;
            repo.set_selected_board(board_id, true).await?;
            state.selected_board_id = Some(board_id);
            screen_manager
                .invalidate_many(&[ScreenType::CurrentSprint, ScreenType::BoardSelection]);
            screen_stack.clear();
            Ok(ScreenState::SwitchTo(ScreenType::Home))
        }
        _ => Ok(ScreenState::Stay),
    }
}

async fn ensure_board_selection_screen<'a>(
    screen_manager: &'a mut ScreenManager,
    cfg_state: &'a AppConfigState,
    state: &'a AppState,
    repo: &'a Option<Arc<RepositoryHub>>,
) -> Result<&'a mut crate::ui::screens::board_selection::BoardSelectionScreen> {
    if screen_manager.board_selection_mut().is_none() {
        let repo = repo.as_ref().ok_or_else(|| {
            color_eyre::eyre::eyre!("Repository not initialized: cannot open boards")
        })?;
        let ctx = ScreenContext {
            cfg_state,
            app_state: state,
            repo: repo.clone(),
        };
        let _ = screen_manager
            .active_screen_mut(ScreenType::BoardSelection, ctx)
            .await?;
    }
    screen_manager
        .board_selection_mut()
        .ok_or_else(|| color_eyre::eyre::eyre!("Board selection screen missing"))
}
