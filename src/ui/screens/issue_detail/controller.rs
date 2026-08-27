use crossterm::event::KeyCode;
use tui_input::InputRequest;

use crate::{
    app::FormPurpose,
    data::IssueMutation,
    ui::{
        interaction::{ActionId, Command, Mode},
        screens::ScreenState,
    },
};

use super::state::IssueDetailState;

pub struct IssueDetailController;

impl IssueDetailController {
    pub fn handle_command(state: &mut IssueDetailState, command: Command) -> ScreenState {
        if state.comment_input().is_some() {
            return Self::handle_comment_command(state, command);
        }
        if state.transition_picker_open() {
            return Self::handle_transition_command(state, command);
        }

        match command.action {
            ActionId::MoveDown => {
                for _ in 0..command.repeat {
                    state.scroll_down();
                }
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                for _ in 0..command.repeat {
                    state.scroll_up();
                }
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                state.scroll_to_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                state.scroll_to_bottom();
                ScreenState::Refresh
            }
            ActionId::PageDown => {
                for _ in 0..command.repeat {
                    state.page_down();
                }
                ScreenState::Refresh
            }
            ActionId::PageUp => {
                for _ in 0..command.repeat {
                    state.page_up();
                }
                ScreenState::Refresh
            }
            ActionId::OpenInBrowser => state
                .browse_url()
                .map(|url| ScreenState::OpenInBrowser(url.to_string()))
                .unwrap_or(ScreenState::Stay),
            ActionId::EditIssue => state
                .issue_key()
                .map(|key| ScreenState::OpenIssueForm(FormPurpose::Edit(key.to_string())))
                .unwrap_or(ScreenState::Stay),
            ActionId::AddComment if state.issue_key().is_some() => {
                state.open_comment_input();
                ScreenState::SwitchMode(Mode::Insert)
            }
            ActionId::TransitionIssue if state.issue_key().is_some() => {
                state.request_transitions();
                ScreenState::Refresh
            }
            ActionId::AssignToMe => state
                .issue_key()
                .map(|key| {
                    ScreenState::Mutate(IssueMutation::AssignToMe {
                        key: key.to_string(),
                    })
                })
                .unwrap_or(ScreenState::Stay),
            ActionId::MoveRight => {
                for _ in 0..command.repeat {
                    state.scroll_right();
                }
                ScreenState::Refresh
            }
            ActionId::MoveLeft => {
                for _ in 0..command.repeat {
                    state.scroll_left();
                }
                ScreenState::Refresh
            }
            ActionId::MoveLineStart => {
                state.reset_horizontal_scroll();
                ScreenState::Refresh
            }
            ActionId::Quit => ScreenState::Close,
            _ => ScreenState::Stay,
        }
    }

    fn handle_comment_command(state: &mut IssueDetailState, command: Command) -> ScreenState {
        match command.action {
            ActionId::Confirm | ActionId::RawInput(KeyCode::Enter) => {
                let Some(key) = state.issue_key().map(str::to_string) else {
                    return ScreenState::Stay;
                };
                state
                    .take_comment()
                    .map(|body| ScreenState::Mutate(IssueMutation::Comment { key, body }))
                    .unwrap_or(ScreenState::Stay)
            }
            ActionId::RawInput(KeyCode::Esc) => {
                state.close_comment_input();
                ScreenState::SwitchMode(Mode::Normal)
            }
            ActionId::RawInput(KeyCode::Char(c)) => {
                state.handle_comment_input(InputRequest::InsertChar(c));
                ScreenState::Refresh
            }
            ActionId::RawInput(KeyCode::Backspace) => {
                state.handle_comment_input(InputRequest::DeletePrevChar);
                ScreenState::Refresh
            }
            ActionId::RawInput(KeyCode::Delete) => {
                state.handle_comment_input(InputRequest::DeleteNextChar);
                ScreenState::Refresh
            }
            ActionId::RawInput(KeyCode::Left) => {
                state.handle_comment_input(InputRequest::GoToPrevChar);
                ScreenState::Refresh
            }
            ActionId::RawInput(KeyCode::Right) => {
                state.handle_comment_input(InputRequest::GoToNextChar);
                ScreenState::Refresh
            }
            ActionId::RawInput(KeyCode::Home) => {
                state.handle_comment_input(InputRequest::GoToStart);
                ScreenState::Refresh
            }
            ActionId::RawInput(KeyCode::End) => {
                state.handle_comment_input(InputRequest::GoToEnd);
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }

    fn handle_transition_command(state: &mut IssueDetailState, command: Command) -> ScreenState {
        match command.action {
            ActionId::MoveUp => {
                for _ in 0..command.repeat {
                    state.select_previous_transition();
                }
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                for _ in 0..command.repeat {
                    state.select_next_transition();
                }
                ScreenState::Refresh
            }
            ActionId::Confirm => {
                let Some(key) = state.issue_key().map(str::to_string) else {
                    return ScreenState::Stay;
                };
                state
                    .selected_transition()
                    .cloned()
                    .map(|transition| {
                        ScreenState::Mutate(IssueMutation::Transition {
                            key,
                            id: transition.id,
                            to_status: transition.to_status,
                        })
                    })
                    .unwrap_or(ScreenState::Stay)
            }
            ActionId::Quit => {
                state.close_transition_picker();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}
