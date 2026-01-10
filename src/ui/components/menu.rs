use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub id: &'static str,
    pub label: String,
    pub hint: Option<String>,
}

impl MenuItem {
    pub fn new(id: &'static str, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
    style: Style,
    highlight_style: Style,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected: 0,
            style: Style::default(),
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
        }
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn set_highlight_style(&mut self, style: Style) {
        self.highlight_style = style;
    }

    pub fn set_items(&mut self, items: Vec<MenuItem>) {
        self.items = items;
        if self.items.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn move_up(&mut self, n: usize) {
        if self.items.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected = self.selected.saturating_sub(step);
    }

    pub fn move_down(&mut self, n: usize) {
        if self.items.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected = (self.selected + step).min(self.items.len() - 1);
    }

    pub fn move_top(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = 0;
    }

    pub fn move_bottom(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = self.items.len() - 1;
    }

    pub fn selected(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    pub fn selected_index(&self) -> Option<usize> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.selected)
        }
    }

    pub fn set_selected_index(&mut self, index: usize) {
        if self.items.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = index.min(self.items.len() - 1);
    }

    pub fn height(&self) -> u16 {
        self.items.len() as u16
    }
}

impl Widget for &Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_hint_width = self
            .items
            .iter()
            .filter_map(|item| item.hint.as_ref())
            .map(|hint| hint.chars().count())
            .max()
            .unwrap_or(0) as u16;
        let max_width = self
            .items
            .iter()
            .map(|item| item_text_width(item, max_hint_width as usize))
            .max()
            .unwrap_or(0) as u16;
        let padding = 2u16;
        let button_width = max_width.saturating_add(padding * 2);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min((area.width.saturating_sub(button_width)) / 2),
                Constraint::Length(button_width.min(area.width)),
                Constraint::Min(0),
            ])
            .split(area);

        let list_items = self
            .items
            .iter()
            .map(|item| ListItem::new(render_line(item, max_hint_width as usize)));

        let mut state = ListState::default();
        if !self.items.is_empty() {
            state.select(Some(self.selected));
        }

        let list = List::new(list_items)
            .style(self.style)
            .highlight_style(self.highlight_style);

        StatefulWidget::render(list, columns[1], buf, &mut state);
    }
}

fn render_line(item: &MenuItem, hint_width: usize) -> Line<'static> {
    match &item.hint {
        Some(hint) => Line::from(vec![
            Span::raw(format!(
                "{:pad$}[{:<width$}] ",
                "",
                hint,
                pad = LEFT_PAD,
                width = hint_width
            )),
            Span::raw(item.label.clone()),
        ]),
        None => Line::from(format!("{:pad$}{}", "", item.label, pad = LEFT_PAD)),
    }
}

fn item_text_width(item: &MenuItem, hint_width: usize) -> usize {
    match &item.hint {
        Some(_) => LEFT_PAD + hint_width + 3 + item.label.chars().count(),
        None => LEFT_PAD + item.label.chars().count(),
    }
}

const LEFT_PAD: usize = 2;
