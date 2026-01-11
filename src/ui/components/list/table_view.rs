use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{
        Block, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};

use crate::ui::context::RenderContext;

pub type TableRow<'a> = Row<'a>;

pub struct TableView<'a> {
    pub rows: &'a [TableRow<'a>],
    pub selected: usize,
    pub context: &'a RenderContext,
}

impl<'a> TableView<'a> {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut state = TableState::default();
        if !self.rows.is_empty() {
            state.select(Some(self.selected.min(self.rows.len().saturating_sub(1))));
        }

        let highlight_style = Style::default()
            .fg(self.context.colors().text)
            .bg(self.context.colors().selection)
            .add_modifier(Modifier::BOLD);

        let base_style = Style::default()
            .fg(self.context.colors().text)
            .bg(self.context.colors().background);
        let block = Block::default()
            .style(base_style)
            .padding(Padding::horizontal(1));

        let table = Table::new(
            self.rows.iter().cloned(),
            [
                Constraint::Length(12),
                Constraint::Fill(1),
                Constraint::Length(14),
            ],
        )
        .header(
            Row::new(["Key", "Summary", "Status"])
                .style(Style::default().fg(self.context.colors().accent)),
        )
        .style(base_style)
        .block(block)
        .row_highlight_style(highlight_style);

        let visible_rows = area.height.saturating_sub(1) as usize;
        let show_scrollbar = visible_rows > 0 && self.rows.len() > visible_rows;
        let table_area = if show_scrollbar {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height,
            }
        } else {
            area
        };

        frame.render_stateful_widget(table, table_area, &mut state);

        if show_scrollbar {
            let mut scrollbar_state = ScrollbarState::new(self.rows.len())
                .position(self.selected.min(self.rows.len().saturating_sub(1)));
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(self.context.colors().accent));
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }
}
