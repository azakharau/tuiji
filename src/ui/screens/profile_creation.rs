use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear},
};

use crate::{
    app::key_handlers::{Command, KeyHandler},
    config::AppConfigState,
    ui::screens::{Screen, ScreenState},
};

struct ProofileFormItem {
    label: &'static str,
    value: String,
    is_password: bool,
    cursor_position: usize,
}

struct ProfileForm {
    items: Vec<ProofileFormItem>,
    selected_index: usize,
}

pub struct ProfileCreationScreen<'a> {
    form: ProfileForm,
    cfg: &'a mut AppConfigState,
}

impl<'a> ProfileCreationScreen<'a> {
    pub fn new(cfg: &'a mut AppConfigState) -> Self {
        let form = ProfileForm {
            items: vec![
                ProofileFormItem {
                    label: "Jira URL",
                    value: String::new(),
                    is_password: false,
                    cursor_position: 0,
                },
                ProofileFormItem {
                    label: "Email",
                    value: String::new(),
                    is_password: false,
                    cursor_position: 0,
                },
                ProofileFormItem {
                    label: "API Token",
                    value: String::new(),
                    is_password: true,
                    cursor_position: 0,
                },
            ],
            selected_index: 0,
        };
        Self { form, cfg }
    }
}

impl Screen for ProfileCreationScreen<'_> {
    fn draw(&mut self, frame: &mut Frame) {
        let [_, vertical_layout] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Fill(1),
            Constraint::Percentage(20),
        ])
        .areas(frame.area());
        let [_, center, _] = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Fill(1),
            Constraint::Percentage(20),
        ])
        .areas(vertical_layout);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = block.inner(center);
        frame.render_widget(Clear, center);
        frame.render_widget(block, center);
    }

    fn name(&self) -> &'static str {
        "Profile Creation"
    }
}

impl KeyHandler for ProfileCreationScreen<'_> {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        // Handle commands specific to profile creation here
        ScreenState::Stay
    }
}
