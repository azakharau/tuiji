use crossterm::event::KeyCode;

use crate::{
    ui::interaction::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::CurrentSprintState;

pub struct CurrentSprintController;

impl CurrentSprintController {
    pub fn handle_command(state: &mut CurrentSprintState, command: Command) -> ScreenState {
        if state.detail_open() {
            return match command.action {
                ActionId::Confirm | ActionId::Quit | ActionId::RawInput(KeyCode::Esc) => {
                    state.close_detail();
                    ScreenState::Refresh
                }
                _ => ScreenState::Stay,
            };
        }

        match command.action {
            ActionId::Confirm => {
                state.toggle_detail();
                ScreenState::Refresh
            }
            ActionId::Refresh => ScreenState::Refresh,
            ActionId::MoveDown => {
                Self::move_down(state, command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                Self::move_up(state, command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                Self::go_top(state);
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                Self::go_bottom(state);
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }

    pub fn update_rows_visible(state: &mut CurrentSprintState, rows_visible: usize) {
        state.set_rows_visible(rows_visible);
        Self::clamp_selection(state);
        Self::ensure_selection_visible(state);
    }

    fn clamp_selection(state: &mut CurrentSprintState) {
        if state.is_empty() {
            state.set_selected_index(0);
            state.set_scroll_offset(0);
            state.close_detail();
        } else if state.selected_index() >= state.issues().len() {
            state.set_selected_index(state.issues().len() - 1);
        }
    }

    fn ensure_selection_visible(state: &mut CurrentSprintState) {
        if state.is_empty() {
            state.set_scroll_offset(0);
            return;
        }

        let rows_visible = state.rows_visible().max(1);
        let mut scroll_offset = state.scroll_offset();
        if state.selected_index() < scroll_offset {
            scroll_offset = state.selected_index();
        } else if state.selected_index() >= scroll_offset + rows_visible {
            scroll_offset = state.selected_index() + 1 - rows_visible;
        }

        let max_offset = state.issues().len().saturating_sub(rows_visible);
        if scroll_offset > max_offset {
            scroll_offset = max_offset;
        }
        state.set_scroll_offset(scroll_offset);
    }

    fn move_down(state: &mut CurrentSprintState, n: usize) {
        if state.is_empty() {
            return;
        }
        let max_index = state.issues().len() - 1;
        state.set_selected_index((state.selected_index() + n.max(1)).min(max_index));
        Self::ensure_selection_visible(state);
    }

    fn move_up(state: &mut CurrentSprintState, n: usize) {
        if state.is_empty() {
            return;
        }
        state.set_selected_index(state.selected_index().saturating_sub(n.max(1)));
        Self::ensure_selection_visible(state);
    }

    fn go_top(state: &mut CurrentSprintState) {
        if state.is_empty() {
            return;
        }
        state.set_selected_index(0);
        state.set_scroll_offset(0);
    }

    fn go_bottom(state: &mut CurrentSprintState) {
        if state.is_empty() {
            return;
        }
        state.set_selected_index(state.issues().len() - 1);
        Self::ensure_selection_visible(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::IssueSummary;

    fn issue(key: &str) -> IssueSummary {
        IssueSummary {
            key: key.to_string(),
            summary: format!("Summary for {key}"),
            epic: None,
            status: "TODO".to_string(),
            issue_type: "Task".to_string(),
            assignee: "Alex".to_string(),
            priority: "Medium".to_string(),
            story_points: None,
            project_key: Some("DEMO".to_string()),
            sprint_id: Some(1),
            updated_at: None,
            comments: Vec::new(),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: None,
            reporter: None,
            creator: None,
            created_at: None,
            resolution_date: None,
            resolution: None,
            labels: Vec::new(),
            fix_versions: Vec::new(),
            parent_key: None,
            environment: None,
            time_estimate: None,
            time_spent: None,
            time_remaining: None,
            custom_fields: None,
        }
    }

    #[test]
    fn confirm_should_toggle_detail_modal() {
        let mut state = CurrentSprintState::new(vec![issue("DEMO-1")]);

        let opened = CurrentSprintController::handle_command(
            &mut state,
            Command {
                action: ActionId::Confirm,
                repeat: 1,
            },
        );
        assert_eq!(opened, ScreenState::Refresh);
        assert!(state.detail_open());

        let closed = CurrentSprintController::handle_command(
            &mut state,
            Command {
                action: ActionId::Confirm,
                repeat: 1,
            },
        );
        assert_eq!(closed, ScreenState::Refresh);
        assert!(!state.detail_open());
    }

    #[test]
    fn move_bottom_should_adjust_scroll_window() {
        let mut state =
            CurrentSprintState::new(vec![issue("1"), issue("2"), issue("3"), issue("4")]);
        CurrentSprintController::update_rows_visible(&mut state, 2);

        let result = CurrentSprintController::handle_command(
            &mut state,
            Command {
                action: ActionId::MoveBottom,
                repeat: 1,
            },
        );

        assert_eq!(result, ScreenState::Refresh);
        assert_eq!(state.selected_index(), 3);
        assert_eq!(state.scroll_offset(), 2);
    }
}
