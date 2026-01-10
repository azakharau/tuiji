use ratatui::widgets::ListItem;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsItemId {
    Profiles,
    Themes,
}

struct SettingsEntry {
    id: SettingsItemId,
    label: &'static str,
}

pub struct SettingsState {
    entries: Vec<SettingsEntry>,
    list_items: Vec<ListItem<'static>>,
    selected_index: usize,
    message: String,
}

impl SettingsState {
    pub fn new() -> Self {
        let entries = vec![
            SettingsEntry {
                id: SettingsItemId::Profiles,
                label: "Profiles",
            },
            SettingsEntry {
                id: SettingsItemId::Themes,
                label: "Themes",
            },
        ];
        let mut state = Self {
            list_items: Vec::new(),
            entries,
            selected_index: 0,
            message: "Enter to open • Esc to close".to_string(),
        };
        state.refresh_items();
        state
    }

    pub fn list_items(&self) -> &[ListItem<'static>] {
        &self.list_items
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn selected_item(&self) -> Option<SettingsItemId> {
        self.entries.get(self.selected_index).map(|entry| entry.id)
    }

    pub fn move_up(&mut self, n: usize) {
        if self.entries.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = self.selected_index.saturating_sub(step);
    }

    pub fn move_down(&mut self, n: usize) {
        if self.entries.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = (self.selected_index + step).min(self.entries.len() - 1);
    }

    pub fn move_top(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn move_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        }
    }

    fn refresh_items(&mut self) {
        let mut items = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            items.push(ListItem::new(entry.label));
        }
        self.list_items = items;
    }
}
