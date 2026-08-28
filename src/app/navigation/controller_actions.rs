use super::*;

impl<'a> NavigationController<'a> {
    pub(crate) fn apply_action(&mut self, action: ScreenState) -> Result<ActionOutcome> {
        match action {
            ScreenState::Quit => Ok(ActionOutcome::Quit),
            ScreenState::SwitchTo(new_screen) => {
                if self.state.current_screen == ScreenType::ProfileCreation
                    && new_screen != ScreenType::ProfileCreation
                {
                    cleanup_profile_creation(self.state, self.screen_manager);
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
            ScreenState::ViewIssue(key) => {
                self.state.issue_detail_key = Some(key);
                self.apply_action(ScreenState::SwitchTo(ScreenType::IssueDetail))
            }
            ScreenState::OpenIssueForm(purpose) => {
                self.state.issue_form_purpose = Some(purpose);
                self.apply_action(ScreenState::SwitchTo(ScreenType::NewIssue))
            }
            ScreenState::Refresh => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::Stay => Ok(ActionOutcome::Continue { render: false }),
            ScreenState::SwitchMode(mode) => {
                self.state.mode = mode;
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::Close => self.close_screen(),
            ScreenState::SaveProfile(_)
            | ScreenState::SaveProfileAndClose(_)
            | ScreenState::ApplyTheme(_)
            | ScreenState::SaveCustomTheme(_)
            | ScreenState::SaveCustomThemeAndClose(_)
            | ScreenState::ResolveConflictLocal(_)
            | ScreenState::ResolveConflictRemote(_)
            | ScreenState::Mutate(_)
            | ScreenState::OpenInBrowser(_)
            | ScreenState::RunSearch(_)
            | ScreenState::SyncNow
            | ScreenState::SyncPause
            | ScreenState::SyncRetry
            | ScreenState::SyncResume => Ok(ActionOutcome::Continue { render: true }),
        }
    }

    pub(crate) fn close_screen(&mut self) -> Result<ActionOutcome> {
        if self.state.current_screen == ScreenType::ProfileCreation {
            cleanup_profile_creation(self.state, self.screen_manager);
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
}
