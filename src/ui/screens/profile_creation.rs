use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, WidgetRef, block::Title},
};

use crate::{
    app::key_handlers::{ActionId, Command, KeyHandler},
    config::AppConfigState,
    ui::screens::{Screen, ScreenState},
};

#[derive(Clone, Debug)]
struct ProfileFormItem {
    label: &'static str,
    value: String,
    is_password: bool,
    cursor_position: usize,
}

#[derive(Clone, Debug)]
struct ProfileForm {
    items: Vec<ProfileFormItem>,
    selected_index: usize,
}

impl Widget for ProfileForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .flex(Flex::Center)
        .split(area);
        let lines = rows[1..rows.len() - 1]
            .iter()
            .map(|r| {
                let [_, line, _] = Layout::horizontal([
                    Constraint::Percentage(5),
                    Constraint::Fill(1),
                    Constraint::Percentage(5),
                ])
                .flex(Flex::Center)
                .areas(*r);
                line
            })
            .collect::<Vec<Rect>>();

        for ((i, item), line) in self.items.iter().enumerate().zip(lines) {
            let mut block = Block::bordered()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Line::from(item.label));

            if self.selected_index == i {
                block = block.style(Style::default().fg(ratatui::style::Color::Cyan));
                let cursor_x = line.x + 1 + item.cursor_position as u16;
                let cursor_y = line.y + 1;
                let rect = Rect {
                    x: cursor_x,
                    y: cursor_y,
                    width: 1,
                    height: 1,
                };
                let cursor_block = Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(ratatui::style::Color::White));
                cursor_block.render(rect, buf);
            }
            let inner_area = block.inner(line);
            block.render(line, buf);
            let display_value = if item.is_password {
                "*".repeat(item.value.len())
            } else {
                item.value.clone()
            };
            let paragraph = Paragraph::new(display_value);
            paragraph.render(inner_area, buf);
        }
    }
}

pub struct ProfileCreationScreen {
    form: ProfileForm,
}

impl ProfileCreationScreen {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ProfileCreationScreen {
    fn default() -> Self {
        let form = ProfileForm {
            items: vec![
                ProfileFormItem {
                    label: "Profile Name",
                    value: String::new(),
                    is_password: false,
                    cursor_position: 0,
                },
                ProfileFormItem {
                    label: "Jira URL",
                    value: String::new(),
                    is_password: false,
                    cursor_position: 0,
                },
                ProfileFormItem {
                    label: "Email",
                    value: String::new(),
                    is_password: false,
                    cursor_position: 0,
                },
                ProfileFormItem {
                    label: "API Token",
                    value: String::new(),
                    is_password: true,
                    cursor_position: 0,
                },
            ],
            selected_index: 0,
        };
        Self { form }
    }
}

impl Screen for ProfileCreationScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let [_, vertical_layout, _] = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Fill(1),
            Constraint::Percentage(30),
        ])
        .areas(frame.area());
        let [_, center, _] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Fill(1),
            Constraint::Percentage(30),
        ])
        .flex(Flex::Center)
        .areas(vertical_layout);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black))
            .title(Line::from(self.name()).centered());

        let body = block.inner(center);
        frame.render_widget(Clear, center);
        frame.render_widget(block, center);
        frame.render_widget(self.form.clone(), body);
    }

    fn name(&self) -> &'static str {
        "Profile Creation"
    }
}

impl KeyHandler for ProfileCreationScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::NextRow => {
                if self.form.selected_index + 1 < self.form.items.len() {
                    self.form.selected_index += 1;
                } else {
                    self.form.selected_index = 0;
                }
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}
