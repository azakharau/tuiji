use ratatui::widgets::ListItem;

use crate::config::ProfileConfig;

struct ListEntry {
    id: &'static str,
    profile_id: Option<String>,
}

pub struct ProfilesState {
    entries: Vec<ListEntry>,
    list_items: Vec<ListItem<'static>>,
    selected_index: usize,
    profile_count: usize,
    message: String,
}

impl ProfilesState {
    pub fn new(profiles: &[ProfileConfig], active_id: Option<&str>) -> Self {
        let (entries, labels, profile_count, message) = build_entries(profiles, active_id);
        let mut state = Self {
            entries,
            list_items: Vec::new(),
            selected_index: 0,
            profile_count,
            message,
        };
        state.refresh_items(&labels);
        state
    }

    pub fn selected_profile_id(&self) -> Option<&str> {
        self.entries
            .get(self.selected_index)
            .and_then(|entry| entry.profile_id.as_deref())
    }

    pub fn is_empty(&self) -> bool {
        self.profile_count == 0
    }

    pub fn selected_menu_id(&self) -> Option<&'static str> {
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

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn list_items(&self) -> &[ListItem<'static>] {
        &self.list_items
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn refresh(
        &mut self,
        profiles: &[ProfileConfig],
        active_id: Option<&str>,
        selected_id: Option<&str>,
    ) {
        let (entries, labels, profile_count, message) = build_entries(profiles, active_id);
        self.entries = entries;
        self.refresh_items(&labels);
        self.profile_count = profile_count;
        self.message = message;
        if let Some(selected_id) = selected_id
            && let Some(idx) = self
                .entries
                .iter()
                .position(|entry| entry.profile_id.as_deref() == Some(selected_id))
        {
            self.selected_index = idx;
        }
    }

    pub fn set_items(&mut self, items: Vec<ListItem<'static>>) {
        self.list_items = items;
    }

    pub fn refresh_items(&mut self, labels: &[String]) {
        let mut items = Vec::with_capacity(labels.len());
        for label in labels {
            items.push(ListItem::new(label.clone()));
        }
        self.set_items(items);
    }
}

fn build_entries(
    profiles: &[ProfileConfig],
    active_id: Option<&str>,
) -> (Vec<ListEntry>, Vec<String>, usize, String) {
    if profiles.is_empty() {
        let entries = vec![
            ListEntry {
                id: "empty",
                profile_id: None,
            },
            ListEntry {
                id: "new",
                profile_id: None,
            },
            ListEntry {
                id: "quit",
                profile_id: None,
            },
        ];
        let labels = vec![
            "No profiles found".to_string(),
            "New profile".to_string(),
            "Quit".to_string(),
        ];
        let message = "No profiles available.\nCreate one to continue.".to_string();
        return (entries, labels, 0, message);
    }

    let mut entries = Vec::with_capacity(profiles.len());
    let mut labels = Vec::with_capacity(profiles.len());
    for profile in profiles {
        let label = if Some(profile.id.as_str()) == active_id {
            format!("{} (active)", profile.name)
        } else {
            profile.name.clone()
        };
        entries.push(ListEntry {
            id: "profile",
            profile_id: Some(profile.id.clone()),
        });
        labels.push(label);
    }

    let message = "Enter to activate • e to edit • d to delete • n to add".to_string();
    (entries, labels, profiles.len(), message)
}
