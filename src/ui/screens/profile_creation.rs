use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use std::sync::Arc;

use crate::{
    app::{
        error::AppErrorState,
        key_handlers::{ActionHint, ActionId, Command, KeyHandler},
        state::Mode,
    },
    config::{JiraConfig, ProfileConfig},
    ui::{
        components::bottom_bar::BottomBar,
        overlays::ErrorModal,
        screens::{CommandLineCommand, Screen, ScreenState},
    },
};

#[derive(Clone, Debug)]
struct ProfileFormItem {
    label: &'static str,
    value: String,
    is_password: bool,
    cursor_position: usize,
}

impl ProfileFormItem {
    fn render_cursor(&self, line: Rect, buf: &mut Buffer) {
        let cursor_x = line.x + 1 + self.cursor_position as u16;
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
    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        let display_value = if self.is_password {
            "*".repeat(self.value.len())
        } else {
            self.value.clone()
        };
        let paragraph = Paragraph::new(display_value);
        paragraph.render(area, buf);
    }
}

#[derive(Clone, Debug)]
struct ProfileForm {
    items: Vec<ProfileFormItem>,
    selected_index: usize,
}

impl ProfileForm {
    fn layut(&self, area: Rect) -> Vec<Rect> {
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

        rows[1..rows.len() - 1]
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
            .collect::<Vec<Rect>>()
    }

    fn go_to_next_row(&mut self) {
        if self.selected_index + 1 < self.items.len() {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    fn go_to_prev_row(&mut self) {
        if self.items.is_empty() {
            self.selected_index = 0;
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    fn go_to_top(&mut self) {
        self.selected_index = 0;
    }

    fn go_to_bottom(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = self.items.len() - 1;
        }
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn word_forward(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    let len = value.len();
    if pos >= len {
        return len;
    }

    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len());

    if idx >= chars.len() {
        return len;
    }

    if is_word_char(chars[idx].1) {
        while idx < chars.len() && is_word_char(chars[idx].1) {
            idx += 1;
        }
    }

    while idx < chars.len() && !is_word_char(chars[idx].1) {
        idx += 1;
    }

    if idx < chars.len() { chars[idx].0 } else { len }
}

fn word_end(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    let len = value.len();
    if pos >= len {
        return len;
    }
    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len().saturating_sub(1));

    if idx >= chars.len() {
        return len;
    }

    if !is_word_char(chars[idx].1) {
        while idx < chars.len() && !is_word_char(chars[idx].1) {
            idx += 1;
        }
    }
    if idx >= chars.len() {
        return len;
    }
    while idx + 1 < chars.len() && is_word_char(chars[idx + 1].1) {
        idx += 1;
    }
    chars[idx].0
}

fn word_backward(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    if pos == 0 {
        return 0;
    }
    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len());
    idx = idx.saturating_sub(1);

    if !is_word_char(chars[idx].1) {
        while idx > 0 && !is_word_char(chars[idx].1) {
            idx -= 1;
        }
        if !is_word_char(chars[idx].1) {
            return 0;
        }
    }

    while idx > 0 && is_word_char(chars[idx - 1].1) {
        idx -= 1;
    }
    chars[idx].0
}

impl Widget for ProfileForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.layut(area);
        for ((i, item), line) in self.items.iter().enumerate().zip(lines) {
            let mut block = Block::bordered()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Line::from(item.label));

            if self.selected_index == i {
                block = block.style(Style::default().fg(Color::Cyan));
                item.render_cursor(line, buf);
            }
            let inner_area = block.inner(line);
            block.render(line, buf);
            item.render_content(inner_area, buf);
        }
    }
}

pub struct ProfileCreationScreen {
    form: ProfileForm,
    actions: Arc<Vec<ActionHint>>,
    mode: Mode,
    profile_id: Option<String>,
    sync_mode: Option<String>,
    error: Option<AppErrorState>,
}

impl ProfileCreationScreen {
    pub fn new(profile: Option<ProfileConfig>) -> Self {
        if let Some(profile) = profile {
            return Self::from_profile(profile);
        }
        Self::default()
    }

    pub fn set_profile_id(&mut self, id: String) {
        self.profile_id = Some(id);
    }

    fn build_profile(&self) -> Result<ProfileConfig, String> {
        let name = self
            .form
            .items
            .get(0)
            .map(|v| v.value.trim().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            return Err("Profile name is required".to_string());
        }

        let jira_url = self
            .form
            .items
            .get(1)
            .map(|v| v.value.trim().to_string())
            .unwrap_or_default();
        validate_url(&jira_url)?;

        let username = self
            .form
            .items
            .get(2)
            .map(|v| v.value.trim().to_string())
            .unwrap_or_default();
        if username.is_empty() {
            return Err("Email is required".to_string());
        }
        validate_email(&username)?;

        let api_token = self
            .form
            .items
            .get(3)
            .map(|v| v.value.trim().to_string())
            .unwrap_or_default();
        if api_token.is_empty() {
            return Err("Jira API token is required".to_string());
        }

        let id = self
            .profile_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        Ok(ProfileConfig {
            id,
            name,
            jira: JiraConfig {
                base_url: jira_url,
                username,
                api_token,
            },
            sync_mode: self.sync_mode.clone(),
        })
    }

    fn from_profile(profile: ProfileConfig) -> Self {
        let mut screen = Self::default();
        screen.profile_id = Some(profile.id);
        screen.sync_mode = profile.sync_mode;
        if let Some(item) = screen.form.items.get_mut(0) {
            item.value = profile.name;
            item.cursor_position = item.value.len();
        }
        if let Some(item) = screen.form.items.get_mut(1) {
            item.value = profile.jira.base_url;
            item.cursor_position = item.value.len();
        }
        if let Some(item) = screen.form.items.get_mut(2) {
            item.value = profile.jira.username;
            item.cursor_position = item.value.len();
        }
        if let Some(item) = screen.form.items.get_mut(3) {
            item.value = profile.jira.api_token;
            item.cursor_position = item.value.len();
        }
        screen
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
        Self {
            form,
            actions: Arc::new(Vec::new()),
            mode: Mode::Normal,
            profile_id: None,
            sync_mode: None,
            error: None,
        }
    }
}

impl Screen for ProfileCreationScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = crate::app::input::overlay::modal_dialog_area(frame.area());
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black))
            .title(Line::from(self.name()).centered());

        let inner = block.inner(area);
        let [form_area, bar_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        frame.render_widget(self.form.clone(), form_area);

        let bottom_bar = BottomBar::new(self.mode, self.actions.clone());
        frame.render_widget(bottom_bar, bar_area);
        if let Some(err) = &self.error {
            frame.render_widget(ErrorModal::new(err), frame.area());
        }
    }

    fn name(&self) -> &'static str {
        if self.profile_id.is_some() {
            "Edit Profile"
        } else {
            "Profile Creation"
        }
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn handle_command_line(&mut self, cmd: CommandLineCommand) -> ScreenState {
        match cmd {
            CommandLineCommand::Write => match self.build_profile() {
                Ok(profile) => ScreenState::SaveProfile(profile),
                Err(err) => {
                    self.error = Some(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::WriteQuit => match self.build_profile() {
                Ok(profile) => ScreenState::SaveProfileAndClose(profile),
                Err(err) => {
                    self.error = Some(AppErrorState::new("Validation Error", err));
                    ScreenState::Refresh
                }
            },
            CommandLineCommand::Quit => ScreenState::Close,
        }
    }
}

impl KeyHandler for ProfileCreationScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        self.error = None;
        match command.action {
            ActionId::MoveDown => {
                for _ in 0..command.repeat {
                    self.form.go_to_next_row();
                }
                ScreenState::Refresh
            }
            ActionId::MoveUp => {
                for _ in 0..command.repeat {
                    self.form.go_to_prev_row();
                }
                ScreenState::Refresh
            }
            ActionId::MoveTop => {
                self.form.go_to_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                self.form.go_to_bottom();
                ScreenState::Refresh
            }
            ActionId::MoveLeft => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index)
                    && item.cursor_position > 0
                {
                    item.cursor_position = item.cursor_position.saturating_sub(command.repeat);
                }
                ScreenState::Refresh
            }
            ActionId::MoveRight => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    let max = item.value.len();
                    item.cursor_position = (item.cursor_position + command.repeat).min(max);
                }
                ScreenState::Refresh
            }
            ActionId::MoveLineStart => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    item.cursor_position = 0;
                }
                ScreenState::Refresh
            }
            ActionId::MoveLineEnd => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    item.cursor_position = item.value.len();
                }
                ScreenState::Refresh
            }
            ActionId::MoveWordForward => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    for _ in 0..command.repeat {
                        item.cursor_position = word_forward(&item.value, item.cursor_position);
                    }
                }
                ScreenState::Refresh
            }
            ActionId::MoveWordBackward => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    for _ in 0..command.repeat {
                        item.cursor_position = word_backward(&item.value, item.cursor_position);
                    }
                }
                ScreenState::Refresh
            }
            ActionId::MoveWordEnd => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    for _ in 0..command.repeat {
                        item.cursor_position = word_end(&item.value, item.cursor_position);
                    }
                }
                ScreenState::Refresh
            }
            ActionId::EnterInsert(mode) => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    match mode {
                        crate::app::key_handlers::InsertMode::Before => {}
                        crate::app::key_handlers::InsertMode::After => {
                            if item.cursor_position < item.value.len() {
                                item.cursor_position += 1;
                            }
                        }
                        crate::app::key_handlers::InsertMode::LineStart => {
                            item.cursor_position = 0;
                        }
                        crate::app::key_handlers::InsertMode::LineEnd => {
                            item.cursor_position = item.value.len();
                        }
                    }
                }
                ScreenState::Refresh
            }
            ActionId::RawInput(c) => {
                if let Some(item) = self.form.items.get_mut(self.form.selected_index) {
                    match c {
                        crossterm::event::KeyCode::Char(ch) => {
                            item.value.insert(item.cursor_position, ch);
                            item.cursor_position += 1;
                        }
                        crossterm::event::KeyCode::Tab => {
                            item.value.insert(item.cursor_position, '\t');
                            item.cursor_position += 1;
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if item.cursor_position > 0 {
                                item.cursor_position -= 1;
                                item.value.remove(item.cursor_position);
                            }
                        }
                        crossterm::event::KeyCode::Delete => {
                            if item.cursor_position < item.value.len() {
                                item.value.remove(item.cursor_position);
                            }
                        }
                        _ => {}
                    }
                }
                ScreenState::Refresh
            }
            _ => ScreenState::Stay,
        }
    }
}

fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("Jira base URL is required".to_string());
    }
    let parsed = reqwest::Url::parse(url).map_err(|_| "Jira base URL is invalid".to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err("Jira base URL must start with http:// or https://".to_string()),
    }
}

fn validate_email(email: &str) -> Result<(), String> {
    if email.chars().any(|c| c.is_whitespace()) {
        return Err("Email must not contain spaces".to_string());
    }
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if local.is_empty() || domain.is_empty() || parts.next().is_some() {
        return Err("Email address is invalid".to_string());
    }
    if domain.starts_with('.') || domain.ends_with('.') || !domain.contains('.') {
        return Err("Email address is invalid".to_string());
    }
    Ok(())
}
