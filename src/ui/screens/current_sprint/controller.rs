use crate::{
    app::key_handlers::{ActionId, Command},
    ui::screens::ScreenState,
};

use super::state::CurrentSprintState;

pub struct CurrentSprintController;

impl CurrentSprintController {
    pub fn handle_command(state: &mut CurrentSprintState, command: Command) -> ScreenState {
        match command.action {
            ActionId::Confirm => {
                if let Some(key) = state.selected_issue_key() {
                    ScreenState::ViewIssue(key.to_string())
                } else {
                    ScreenState::Stay
                }
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
            ActionId::MoveLeft => {
                Self::move_left(state, command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveRight => {
                Self::move_right(state, command.repeat);
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
        Self::ensure_valid_column(state);
        Self::clamp_selection(state);
        Self::ensure_selection_visible(state);
    }

    fn column_counts(state: &CurrentSprintState) -> Vec<usize> {
        let mut counts = vec![0; state.board_cfg().columns.len()];
        for issue in state.issues().iter() {
            if let Some(idx) = state
                .board_cfg()
                .columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(&issue.status))
            {
                counts[idx] += 1;
            }
        }
        counts
    }

    fn ensure_valid_column(state: &mut CurrentSprintState) {
        let len = state.board_cfg().columns.len();
        if len == 0 {
            state.set_selected_col(0);
            state.set_selected_row(0);
            state.set_scroll_offset(0);
            return;
        }
        if state.selected_col() >= len {
            state.set_selected_col(len - 1);
        }
        let counts = Self::column_counts(state);
        if counts.get(state.selected_col()).copied().unwrap_or(0) > 0 {
            return;
        }
        if let Some(left) = (0..=state.selected_col())
            .rev()
            .find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            state.set_selected_col(left);
        } else if let Some(right) =
            (state.selected_col() + 1..len).find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            state.set_selected_col(right);
        } else {
            state.set_selected_col(0);
            state.set_selected_row(0);
            state.set_scroll_offset(0);
        }
    }

    fn clamp_selection(state: &mut CurrentSprintState) {
        let counts = Self::column_counts(state);
        let col_count = counts.get(state.selected_col()).copied().unwrap_or(0);
        if col_count == 0 {
            state.set_selected_row(0);
        } else if state.selected_row() >= col_count {
            state.set_selected_row(col_count - 1);
        }
    }

    fn ensure_selection_visible(state: &mut CurrentSprintState) {
        let counts = Self::column_counts(state);
        let col_count = counts.get(state.selected_col()).copied().unwrap_or(0);
        let rows_visible = state.rows_visible().max(1);
        if col_count <= rows_visible {
            state.set_scroll_offset(0);
            return;
        }
        let mut scroll_offset = state.scroll_offset();
        if state.selected_row() < scroll_offset {
            scroll_offset = state.selected_row();
        } else if state.selected_row() >= scroll_offset + rows_visible {
            scroll_offset = state.selected_row() + 1 - rows_visible;
        }
        let max_offset = counts
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .saturating_sub(rows_visible);
        if scroll_offset > max_offset {
            scroll_offset = max_offset;
        }
        state.set_scroll_offset(scroll_offset);
    }

    fn move_down(state: &mut CurrentSprintState, n: usize) {
        let counts = Self::column_counts(state);
        let count = counts.get(state.selected_col()).copied().unwrap_or(0);
        if count == 0 {
            return;
        }
        let max_row = count - 1;
        state.set_selected_row((state.selected_row() + n).min(max_row));
        Self::ensure_selection_visible(state);
    }

    fn move_up(state: &mut CurrentSprintState, n: usize) {
        if state.selected_row() > 0 {
            state.set_selected_row(state.selected_row().saturating_sub(n));
        }
        Self::ensure_selection_visible(state);
    }

    fn move_left(state: &mut CurrentSprintState, steps: usize) {
        if state.board_cfg().columns.is_empty() {
            return;
        }
        let counts = Self::column_counts(state);
        let target = state.selected_col().saturating_sub(steps);
        if counts.get(target).copied().unwrap_or(0) > 0 {
            state.set_selected_col(target);
        } else if target > 0
            && let Some(idx) = (0..target)
                .rev()
                .find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            state.set_selected_col(idx);
        }
        Self::clamp_selection(state);
        Self::ensure_selection_visible(state);
    }

    fn move_right(state: &mut CurrentSprintState, steps: usize) {
        if state.board_cfg().columns.is_empty() {
            return;
        }
        let counts = Self::column_counts(state);
        let len = state.board_cfg().columns.len();
        let target = (state.selected_col() + steps).min(len.saturating_sub(1));
        if counts.get(target).copied().unwrap_or(0) > 0 {
            state.set_selected_col(target);
        } else if target + 1 < len
            && let Some(idx) = (target + 1..len).find(|&i| counts.get(i).copied().unwrap_or(0) > 0)
        {
            state.set_selected_col(idx);
        }
        Self::clamp_selection(state);
        Self::ensure_selection_visible(state);
    }

    fn go_top(state: &mut CurrentSprintState) {
        state.set_selected_row(0);
        state.set_scroll_offset(0);
    }

    fn go_bottom(state: &mut CurrentSprintState) {
        let counts = Self::column_counts(state);
        if let Some(max_rows) = counts.get(state.selected_col())
            && *max_rows > 0
        {
            state.set_selected_row(*max_rows - 1);
            Self::ensure_selection_visible(state);
        }
    }
}
