use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, WidgetRef, block::Title},
};

use crate::{
    app::key_handlers::{Command, KeyHandler},
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
            Constraint::Min(5),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(5),
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

        for (item, line) in self.items.iter().zip(lines) {
            let block = Block::bordered()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Line::from(item.label));
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
            Constraint::Percentage(20),
            Constraint::Fill(1),
            Constraint::Percentage(20),
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
            .title(Line::from("Create New Profile").centered());

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
        // Handle commands specific to profile creation here
        ScreenState::Stay
    }
}
