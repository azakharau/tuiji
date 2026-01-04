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
    config::ProfileConfig,
    ui::{
        components::{
            bottom_bar::BottomBar,
            menu::{Menu, MenuItem},
        },
        screens::{Screen, ScreenState},
    },
};

pub struct ProfilesScreen {
    mode: Mode,
    actions: Arc<Vec<ActionHint>>,
    menu: Menu,
    profile_ids: Vec<String>,
    message: String,
}

impl ProfilesScreen {
    pub fn new(profiles: &[ProfileConfig], active_id: Option<&str>) -> Self {
        let (menu, profile_ids, message) = build_menu(profiles, active_id);
        Self {
            mode: Mode::Normal,
            actions: Arc::new(Vec::new()),
            menu,
            profile_ids,
            message,
        }
    }

    pub fn selected_profile_id(&self) -> Option<&str> {
        self.menu
            .selected_index()
            .and_then(|idx| self.profile_ids.get(idx).map(|v| v.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.profile_ids.is_empty()
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

    pub fn selected_menu_id(&self) -> Option<&'static str> {
        self.menu.selected().map(|item| item.id)
    }
}

impl Screen for ProfilesScreen {
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
        "Profiles"
    }

    fn set_action_hints(&mut self, actions: Arc<Vec<ActionHint>>) {
        self.actions = actions;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl KeyHandler for ProfilesScreen {
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

fn build_menu(profiles: &[ProfileConfig], active_id: Option<&str>) -> (Menu, Vec<String>, String) {
    if profiles.is_empty() {
        let menu = Menu::new(vec![
            MenuItem::new("empty", "No profiles found"),
            MenuItem::new("new", "New profile").with_hint("n"),
            MenuItem::new("quit", "Quit").with_hint("q"),
        ]);
        let message = "No profiles available.\nCreate one to continue.".to_string();
        return (menu, Vec::new(), message);
    }

    let mut items = Vec::with_capacity(profiles.len());
    let mut ids = Vec::with_capacity(profiles.len());
    for profile in profiles {
        let label = if Some(profile.id.as_str()) == active_id {
            format!("{} (active)", profile.name)
        } else {
            profile.name.clone()
        };
        items.push(MenuItem::new("profile", label));
        ids.push(profile.id.clone());
    }

    let message = "Enter to activate • e to edit • d to delete • n to add".to_string();
    (Menu::new(items), ids, message)
}
