use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::ui::context::RenderContext;

pub struct ListView<'a> {
    pub items: &'a [ListItem<'a>],
    pub selected: usize,
    pub context: &'a RenderContext,
}

impl<'a> ListView<'a> {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut state = ListState::default();
        if !self.items.is_empty() {
            state.select(Some(self.selected.min(self.items.len().saturating_sub(1))));
        }

        let highlight_style = Style::default()
            .fg(self.context.colors().text)
            .bg(self.context.colors().selection)
            .add_modifier(Modifier::BOLD);

        let base_style = Style::default()
            .fg(self.context.colors().text)
            .bg(self.context.colors().background);
        let list = List::new(self.items.iter().cloned())
            .style(base_style)
            .highlight_style(highlight_style);
        frame.render_stateful_widget(list, area, &mut state);

        if self.items.len() > area.height as usize {
            let mut scrollbar_state = ScrollbarState::new(self.items.len())
                .position(self.selected.min(self.items.len().saturating_sub(1)));
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(self.context.colors().accent));
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }
}
