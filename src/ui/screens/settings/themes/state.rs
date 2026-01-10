use ratatui::widgets::ListItem;

use crate::{config::CustomThemeConfig, ui::theme::ThemeRegistry};

#[derive(Clone, Debug)]
enum ThemeEntry {
    Theme { id: String, name: String },
    Create,
}

pub struct SettingsThemesState {
    entries: Vec<ThemeEntry>,
    list_items: Vec<ListItem<'static>>,
    selected_index: usize,
    active_theme_id: String,
    message: String,
}

impl SettingsThemesState {
    pub fn new(active_theme: &str, custom_themes: &[CustomThemeConfig]) -> Self {
        let themes = ThemeRegistry::themes_with_custom(custom_themes);
        let active_theme_id = if custom_themes.iter().any(|t| t.id == active_theme) {
            active_theme.to_string()
        } else {
            ThemeRegistry::resolve_id(active_theme).to_string()
        };
        let mut entries = themes
            .into_iter()
            .map(|theme| ThemeEntry::Theme {
                id: theme.id,
                name: theme.name,
            })
            .collect::<Vec<_>>();
        entries.push(ThemeEntry::Create);
        let selected_index = entries
            .iter()
            .position(|entry| matches!(entry, ThemeEntry::Theme { id, .. } if id == &active_theme_id))
            .unwrap_or(0);
        let mut state = Self {
            entries,
            list_items: Vec::new(),
            selected_index,
            active_theme_id,
            message: "Enter to apply • Create custom at bottom".to_string(),
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

    pub fn selected_theme_id(&self) -> Option<&str> {
        match self.entries.get(self.selected_index) {
            Some(ThemeEntry::Theme { id, .. }) => Some(id.as_str()),
            _ => None,
        }
    }

    pub fn selected_is_create(&self) -> bool {
        matches!(
            self.entries.get(self.selected_index),
            Some(ThemeEntry::Create)
        )
    }

    pub fn set_active_theme(&mut self, theme_id: &str) {
        self.active_theme_id = theme_id.to_string();
        self.refresh_items();
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
            match entry {
                ThemeEntry::Theme { id, name } => {
                    let label = if id == &self.active_theme_id {
                        format!("{} (active)", name)
                    } else {
                        name.clone()
                    };
                    items.push(ListItem::new(label));
                }
                ThemeEntry::Create => {
                    items.push(ListItem::new("Create custom theme"));
                }
            }
        }
        self.list_items = items;
    }
}
