use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::Text,
    widgets::{ListItem, Paragraph, Wrap},
};

use crate::{
    app::input::overlay::modal_dialog_area,
    app::{
        key_handlers::{ActionHint, ActionId, Command, KeyHandler},
        state::Mode,
    },
    data::BoardSummary,
    ui::{
        components::{
            bottom_bar::BottomBar,
            list::{EmptyState, ListView},
        },
        context::RenderContext,
        screens::{Screen, ScreenState},
    },
};

pub struct BoardSelectionScreen {
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
    list_items: Vec<ListItem<'static>>,
    board_ids: Vec<u64>,
    selected_index: usize,
    message: String,
}

const LIST_SIDE_PADDING: u16 = 15;

impl BoardSelectionScreen {
    pub fn new(boards: Vec<BoardSummary>) -> Self {
        let (labels, board_ids, message) = build_entries(boards);
        let mut screen = Self {
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
            list_items: Vec::new(),
            board_ids,
            selected_index: 0,
            message,
        };
        screen.refresh_items(&labels);
        screen
    }
}

impl Screen for BoardSelectionScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        let area = modal_dialog_area(frame.area());
        let block = ratatui::widgets::Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(ratatui::style::Style::default().fg(context.colors().border))
            .style(
                ratatui::style::Style::default()
                    .fg(context.colors().text)
                    .bg(context.colors().background),
            )
            .title(ratatui::text::Line::from(self.name()).centered());
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(&block, area);
        let inner = block.inner(area);
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(inner);
        let list_area = Layout::horizontal([
            Constraint::Length(LIST_SIDE_PADDING),
            Constraint::Fill(1),
            Constraint::Length(LIST_SIDE_PADDING),
        ])
        .split(layout[1])[1];
        let text = Paragraph::new(Text::from(self.message.as_str()))
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        frame.render_widget(text, layout[0]);
        if self.list_items.is_empty() {
            frame.render_widget(
                EmptyState::new("No boards", "Sync or configure boards first.", &context),
                layout[1],
            );
        } else {
            let list_view = ListView {
                items: &self.list_items,
                selected: self.selected_index,
                context,
            };
            list_view.render(frame, list_area);
        }
        let bottom_bar = BottomBar::new(self.mode, self.actions.clone());
        frame.render_widget(bottom_bar, layout[3]);
    }

    fn name(&self) -> &'static str {
        "Board Selection"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl KeyHandler for BoardSelectionScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::MoveUp => {
                self.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                self.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                self.move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                self.move_bottom();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}

impl BoardSelectionScreen {
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
    }

    pub fn refresh_items(&mut self, labels: &[String]) {
        let mut items = Vec::with_capacity(labels.len());
        for label in labels {
            items.push(ListItem::new(label.clone()));
        }
        self.set_items(items);
    }
}

fn build_entries(boards: Vec<BoardSummary>) -> (Vec<String>, Vec<u64>, String) {
    if boards.is_empty() {
        let message = "No boards are available yet.\nSync or configure boards first.".to_string();
        return (Vec::new(), Vec::new(), message);
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
