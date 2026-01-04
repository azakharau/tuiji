use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::Text,
    widgets::{Paragraph, Wrap},
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
            menu::{Menu, MenuItem},
        },
        screens::{Screen, ScreenState},
    },
};

pub struct BoardSelectionScreen {
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
    menu: Menu,
    board_ids: Vec<u64>,
    message: String,
}

impl BoardSelectionScreen {
    pub fn new(boards: Vec<BoardSummary>) -> Self {
        let (menu, board_ids, message) = build_menu(boards);
        Self {
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
            menu,
            board_ids,
            message,
        }
    }
}

impl Screen for BoardSelectionScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = modal_dialog_area(frame.area());
        let block = ratatui::widgets::Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(ratatui::text::Line::from(self.name()).centered());
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(&block, area);
        let inner = block.inner(area);
        let menu_height = self.menu.height().min(inner.height.saturating_sub(3));
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(menu_height),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(inner);
        let text = Paragraph::new(Text::from(self.message.clone()))
            .alignment(Alignment::Center)
            .wrap(Wrap::default());
        frame.render_widget(text, layout[0]);
        frame.render_widget(&self.menu, layout[1]);
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
                self.menu.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                self.menu.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                self.menu.move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                self.menu.move_bottom();
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}

impl BoardSelectionScreen {
    pub fn selected_board_id(&self) -> Option<u64> {
        self.menu
            .selected_index()
            .and_then(|idx| self.board_ids.get(idx).copied())
    }

    pub fn move_up(&mut self, n: usize) {
        self.menu.move_up(n);
    }

    pub fn move_down(&mut self, n: usize) {
        self.menu.move_down(n);
    }

    pub fn move_top(&mut self) {
        self.menu.move_top();
    }

    pub fn move_bottom(&mut self) {
        self.menu.move_bottom();
    }
}

fn build_menu(boards: Vec<BoardSummary>) -> (Menu, Vec<u64>, String) {
    if boards.is_empty() {
        let menu = Menu::new(vec![
            MenuItem::new("empty", "No boards found"),
            MenuItem::new("quit", "Quit").with_hint("q"),
        ]);
        let message = "No boards are available yet.\nSync or configure boards first.".to_string();
        return (menu, Vec::new(), message);
    }

    let mut items = Vec::with_capacity(boards.len());
    let mut ids = Vec::with_capacity(boards.len());
    for board in boards {
        let label = match board.type_name.as_deref() {
            Some(t) => format!("{} ({})", board.name, t),
            None => board.name,
        };
        items.push(MenuItem::new("board", label));
        ids.push(board.id);
    }

    let message = "Select a board to continue".to_string();
    (Menu::new(items), ids, message)
}
