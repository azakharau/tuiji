use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Borders, Clear},
};

use crate::{
    app::key_handlers::{Command, KeyHandler},
    config::AppConfigState,
    ui::screens::{Screen, ScreenState},
};

struct ProfileFormItem {
    label: &'static str,
    value: String,
    is_password: bool,
    cursor_position: usize,
}

struct ProfileForm {
    items: Vec<ProfileFormItem>,
    selected_index: usize,
}

pub struct ProfileCreationScreen {
    form: ProfileForm,
}

impl ProfileCreationScreen {
    pub fn new() -> Self {
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

impl KeyHandler for ProfileCreationScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        // Handle commands specific to profile creation here
        ScreenState::Stay
    }
}
