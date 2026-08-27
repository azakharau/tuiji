use ratatui::widgets::ListItem;

use crate::data::BoardSummary;

pub struct BoardSelectionState {
    list_items: Vec<ListItem<'static>>,
    board_ids: Vec<u64>,
    selected_index: usize,
    message: String,
}

impl BoardSelectionState {
    pub fn new(boards: Vec<BoardSummary>) -> Self {
        let (labels, board_ids, message) = build_entries(boards);
        let list_items = build_list_items(&labels);
        Self {
            list_items,
            board_ids,
            selected_index: 0,
            message,
        }
    }

    pub fn list_items(&self) -> &[ListItem<'static>] {
        &self.list_items
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.list_items.is_empty()
    }

    pub fn selected_board_id(&self) -> Option<u64> {
        self.board_ids.get(self.selected_index).copied()
    }

    pub fn move_up(&mut self, n: usize) {
        if self.list_items.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = self.selected_index.saturating_sub(step);
    }

    pub fn move_down(&mut self, n: usize) {
        if self.list_items.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = (self.selected_index + step).min(self.list_items.len() - 1);
    }

    pub fn move_top(&mut self) {
        if !self.list_items.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn move_bottom(&mut self) {
        if !self.list_items.is_empty() {
            self.selected_index = self.list_items.len() - 1;
        }
    }

    pub fn set_items(&mut self, items: Vec<ListItem<'static>>) {
        self.list_items = items;
        if self.list_items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.list_items.len() {
            self.selected_index = self.list_items.len() - 1;
        }
    }

    pub fn refresh_items(&mut self, labels: &[String]) {
        self.set_items(build_list_items(labels));
    }
}

fn build_list_items(labels: &[String]) -> Vec<ListItem<'static>> {
    let mut items = Vec::with_capacity(labels.len());
    for label in labels {
        items.push(ListItem::new(label.clone()));
    }
    items
}

fn build_entries(boards: Vec<BoardSummary>) -> (Vec<String>, Vec<u64>, String) {
    if boards.is_empty() {
        return (Vec::new(), Vec::new(), String::new());
    }

    let mut labels = Vec::with_capacity(boards.len());
    let mut ids = Vec::with_capacity(boards.len());
    for board in boards {
        let label = match board.type_name.as_deref() {
            Some(t) => format!("{} ({})", board.name, t),
            None => board.name,
        };
        labels.push(label);
        ids.push(board.id);
    }

    let message = "Select a board to continue".to_string();
    (labels, ids, message)
}
