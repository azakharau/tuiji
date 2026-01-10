use std::sync::Arc;

use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{Paragraph, Row},
};

use crate::{
    app::{
        key_handlers::{ActionHint, ActionId, Command, KeyHandler},
        state::Mode,
    },
    data::{AppRepository, IssueSummary},
    ui::{
        components::{
            bottom_bar::BottomBar,
            list::{EmptyState, TableRow, TableView},
        },
        context::RenderContext,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

pub struct MyIssuesScreen {
    rows: Vec<TableRow<'static>>,
    selected_index: usize,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
}

impl MyIssuesScreen {
    pub async fn new(repo: Arc<dyn AppRepository>, mode: Mode, board_id: u64) -> Result<Self> {
        let issues = repo.current_sprint_issues(board_id).await?;
        Ok(Self {
            rows: build_rows(issues),
            selected_index: 0,
            actions: Arc::new(Vec::new()),
            mode,
        })
    }
}

impl Screen for MyIssuesScreen {
    fn draw(&mut self, frame: &mut Frame, context: &RenderContext) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(ratatui::widgets::Block::default().style(base_style), frame.area());
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let title = Paragraph::new(Line::from(self.name()).centered())
            .style(Style::default().fg(context.colors().accent))
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        if self.rows.is_empty() {
            frame.render_widget(
                EmptyState::new("No issues", "Sync to load issues.", context),
                layout[1],
            );
        } else {
            let table = TableView {
                rows: &self.rows,
                selected: self.selected_index,
                context,
            };
            table.render(frame, layout[1]);
        }

        let bottom_bar = BottomBar::new(self.mode, self.actions.clone());
        frame.render_widget(bottom_bar, layout[2]);
    }

    fn name(&self) -> &'static str {
        "My Issues"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => ScreenState::Stay,
            CommandLineCommand::WriteQuit => ScreenState::Close,
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

impl KeyHandler for MyIssuesScreen {
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

impl MyIssuesScreen {
    fn move_up(&mut self, n: usize) {
        if self.rows.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = self.selected_index.saturating_sub(step);
    }

    fn move_down(&mut self, n: usize) {
        if self.rows.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = (self.selected_index + step).min(self.rows.len() - 1);
    }

    fn move_top(&mut self) {
        if !self.rows.is_empty() {
            self.selected_index = 0;
        }
    }

    fn move_bottom(&mut self) {
        if !self.rows.is_empty() {
            self.selected_index = self.rows.len() - 1;
        }
    }
}

fn build_rows(issues: Vec<IssueSummary>) -> Vec<TableRow<'static>> {
    let mut rows = Vec::with_capacity(issues.len());
    for issue in issues {
        let IssueSummary { key, summary, status, .. } = issue;
        rows.push(Row::new(vec![key, summary, status]));
    }
    rows
}
