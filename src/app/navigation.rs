use crate::{
    app::key_handlers::KeyBindings,
    app::{
        ActionOutcome, AppState, ProfileEditorIntent, render::RenderStack,
        screen_manager::ScreenManager, state::ScreenType,
    },
    config::AppConfigState,
    ui::interaction::BoardRequiredBindings,
    ui::screens::ScreenState,
};
use color_eyre::Result;

mod board_required;
mod close_policy;
mod controller_actions;
mod modal;
mod render_policy;

pub use board_required::{
    board_required_active, board_required_bindings, is_board_required_screen,
};
pub use modal::{has_modal_stack, is_modal_screen};
pub use render_policy::build_render_stack;

use close_policy::{close_all_modals_target, should_cleanup_profile_creation};

pub struct NavigationController<'a> {
    state: &'a mut AppState,
    screen_stack: &'a mut Vec<ScreenType>,
    screen_manager: &'a mut ScreenManager,
    cfg_state: &'a AppConfigState,
    key_bindings: &'a KeyBindings,
}

impl<'a> NavigationController<'a> {
    pub fn new(
        state: &'a mut AppState,
        screen_stack: &'a mut Vec<ScreenType>,
        screen_manager: &'a mut ScreenManager,
        cfg_state: &'a AppConfigState,
        key_bindings: &'a KeyBindings,
    ) -> Self {
        Self {
            state,
            screen_stack,
            screen_manager,
            cfg_state,
            key_bindings,
        }
    }

    pub fn close_all_modals(&mut self) -> Result<ScreenState> {
        close_all_modals_impl(self.state, self.screen_stack, self.screen_manager)
    }

    pub fn build_render_stack(&self) -> RenderStack<'_> {
        build_render_stack(self.state.current_screen, self.screen_stack)
    }

    pub fn is_modal_screen(&self, screen: ScreenType) -> bool {
        is_modal_screen(screen)
    }

    pub fn has_modal_stack(&self) -> bool {
        has_modal_stack(self.state.current_screen, self.screen_stack)
    }

    pub fn board_required_active(&self) -> bool {
        board_required_active(self.state)
    }

    pub fn is_board_required_screen(&self, screen: ScreenType) -> bool {
        is_board_required_screen(screen)
    }

    pub fn board_required_bindings(&self) -> BoardRequiredBindings<'_> {
        board_required_bindings(self.state.current_screen, self.key_bindings)
    }

    pub fn cfg_state(&self) -> &AppConfigState {
        self.cfg_state
    }
}

pub(crate) fn close_all_modals_impl(
    state: &mut AppState,
    screen_stack: &mut Vec<ScreenType>,
    screen_manager: &mut ScreenManager,
) -> Result<ScreenState> {
    let Some(target) = close_all_modals_target(state.current_screen, screen_stack) else {
        return Ok(ScreenState::Stay);
    };

    if should_cleanup_profile_creation(state.current_screen, screen_stack) {
        cleanup_profile_creation(state, screen_manager);
    } else if is_modal_screen(state.current_screen) {
        state.profile_editor = None;
    }
    screen_stack.clear();
    state.current_screen = target;
    Ok(ScreenState::Refresh)
}

pub(super) fn cleanup_profile_creation(state: &mut AppState, screen_manager: &mut ScreenManager) {
    screen_manager.invalidate(ScreenType::ProfileCreation);
    state.profile_editor = None;
}
