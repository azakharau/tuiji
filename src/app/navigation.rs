use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::{
    app::key_handlers::KeyBindings,
    app::{
        ActionOutcome, AppState, ProfileEditorIntent, overlay::BoardRequiredBindings,
        render::RenderStack, screen_manager::ScreenManager, state::ScreenType,
    },
    config::AppConfigState,
    ui::screens::ScreenState,
};

pub fn is_modal_screen(screen: ScreenType) -> bool {
    matches!(
        screen,
        ScreenType::Profiles
            | ScreenType::ProfileCreation
            | ScreenType::BoardSelection
            | ScreenType::Settings
            | ScreenType::SettingsThemes
            | ScreenType::SettingsThemeForm
    )
}

pub fn has_modal_stack(current_screen: ScreenType, screen_stack: &[ScreenType]) -> bool {
    is_modal_screen(current_screen) || screen_stack.iter().any(|screen| is_modal_screen(*screen))
}

pub fn is_board_required_screen(screen: ScreenType) -> bool {
    matches!(
        screen,
        ScreenType::Home
            | ScreenType::CurrentSprint
            | ScreenType::MyIssues
            | ScreenType::SearchIssues
    )
}

pub fn board_required_active(state: &AppState) -> bool {
    state.selected_board_id.is_none() && is_board_required_screen(state.current_screen)
}

pub fn build_render_stack(
    current_screen: ScreenType,
    screen_stack: &[ScreenType],
) -> RenderStack<'_> {
    let include_stack = matches!(
        current_screen,
        ScreenType::Profiles
            | ScreenType::ProfileCreation
            | ScreenType::BoardSelection
            | ScreenType::Settings
            | ScreenType::SettingsThemes
            | ScreenType::SettingsThemeForm
    );
    RenderStack::new(current_screen, screen_stack, include_stack)
}

pub fn board_required_bindings<'a>(
    current_screen: ScreenType,
    key_bindings: &'a KeyBindings,
) -> BoardRequiredBindings<'a> {
    let bindings = key_bindings.bindings_for_screen_ref(current_screen);
    let open_key = bindings
        .iter()
        .find(|entry| entry.action == crate::app::key_handlers::ActionId::OpenBoards)
        .map(|entry| entry.binding.as_str())
        .unwrap_or("b");
    let profiles_key = bindings
        .iter()
        .find(|entry| entry.action == crate::app::key_handlers::ActionId::OpenProfiles)
        .map(|entry| entry.binding.as_str());
    let quit_key = bindings
        .iter()
        .find(|entry| entry.action == crate::app::key_handlers::ActionId::Quit)
        .map(|entry| entry.binding.as_str())
        .unwrap_or("q");
    BoardRequiredBindings {
        open: open_key,
        profiles: profiles_key,
        quit: quit_key,
    }
}

pub(crate) fn close_all_modals_impl(
    state: &mut AppState,
    screen_stack: &mut Vec<ScreenType>,
    _terminal: &mut DefaultTerminal,
) -> Result<ScreenState> {
    if !has_modal_stack(state.current_screen, screen_stack) {
        return Ok(ScreenState::Stay);
    }
    let target = screen_stack
        .iter()
        .rev()
        .find(|screen| !is_modal_screen(**screen))
        .copied()
        .unwrap_or(ScreenType::Home);
    screen_stack.clear();
    if is_modal_screen(state.current_screen) {
        state.profile_editor = None;
    }
    state.current_screen = target;
    Ok(ScreenState::Refresh)
}

pub struct NavigationController<'a> {
    state: &'a mut AppState,
    screen_stack: &'a mut Vec<ScreenType>,
    screen_manager: &'a mut ScreenManager,
    terminal: &'a mut DefaultTerminal,
    cfg_state: &'a AppConfigState,
    key_bindings: &'a KeyBindings,
}

impl<'a> NavigationController<'a> {
    pub fn new(
        state: &'a mut AppState,
        screen_stack: &'a mut Vec<ScreenType>,
        screen_manager: &'a mut ScreenManager,
        terminal: &'a mut DefaultTerminal,
        cfg_state: &'a AppConfigState,
        key_bindings: &'a KeyBindings,
    ) -> Self {
        Self {
            state,
            screen_stack,
            screen_manager,
            terminal,
            cfg_state,
            key_bindings,
        }
    }

    pub(crate) fn apply_action(&mut self, action: ScreenState) -> Result<ActionOutcome> {
        match action {
            ScreenState::Quit => {
                self.terminal.clear()?;
                Ok(ActionOutcome::Quit)
            }
            ScreenState::SwitchTo(new_screen) => {
                if self.state.current_screen == ScreenType::ProfileCreation
                    && new_screen != ScreenType::ProfileCreation
                {
                    self.screen_manager.invalidate(ScreenType::ProfileCreation);
                    self.state.profile_editor = None;
                }
                if new_screen == ScreenType::Home {
                    self.screen_stack.clear();
                } else if new_screen != self.state.current_screen {
                    self.screen_stack.push(self.state.current_screen);
                }
                self.state.current_screen = new_screen;
                if new_screen == ScreenType::ProfileCreation && self.state.profile_editor.is_none()
                {
                    self.state.profile_editor = Some(ProfileEditorIntent::New);
                }
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::Refresh => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::Stay => Ok(ActionOutcome::Continue { render: false }),
            ScreenState::SwitchMode(mode) => {
                self.state.mode = mode;
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::Close => self.close_screen(),
            ScreenState::ViewIssue(_) => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::OpenInBrowser(_) => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::SaveProfile(_)
            | ScreenState::SaveProfileAndClose(_)
            | ScreenState::ApplyTheme(_)
            | ScreenState::SaveCustomTheme(_)
            | ScreenState::SaveCustomThemeAndClose(_)
            | ScreenState::ResolveConflictLocal(_)
            | ScreenState::ResolveConflictRemote(_)
            | ScreenState::CreateIssue(_)
            | ScreenState::SyncNow
            | ScreenState::SyncPause
            | ScreenState::SyncRetry
            | ScreenState::SyncResume => Ok(ActionOutcome::Continue { render: true }),
        }
    }

    pub(crate) fn close_screen(&mut self) -> Result<ActionOutcome> {
        if self.state.current_screen == ScreenType::ProfileCreation {
            self.screen_manager.invalidate(ScreenType::ProfileCreation);
            self.state.profile_editor = None;
        }
        if let Some(prev) = self.screen_stack.pop() {
            self.state.current_screen = prev;
            Ok(ActionOutcome::Continue { render: true })
        } else if is_modal_screen(self.state.current_screen) {
            self.state.current_screen = ScreenType::Home;
            Ok(ActionOutcome::Continue { render: true })
        } else {
            Ok(ActionOutcome::Quit)
        }
    }

    pub fn close_all_modals(&mut self) -> Result<ScreenState> {
        close_all_modals_impl(self.state, self.screen_stack, self.terminal)
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
