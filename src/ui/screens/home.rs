use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    app::{
        key_handlers::{ActionId, Command, KeyHandler},
        state::ScreenType,
    },
    config::AppConfigState,
    ui::{
        components::{
            logo::AsciiLogoComponent,
            menu::{Menu, MenuItem},
        },
        screens::{Screen, ScreenState},
    },
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
enum HomeVariant {
    Welcome,
    #[default]
    Default,
}

#[derive(Debug, Clone)]
pub struct HomeScreen {
    logo: AsciiLogoComponent,
    menu: Menu,
    variant: HomeVariant,
    welcome_text: String,
}

impl HomeScreen {
    pub fn new(logo: AsciiLogoComponent, cfg: &AppConfigState) -> Self {
        let variant = match cfg {
            AppConfigState::Loaded(_) => HomeVariant::Default,
            AppConfigState::Missing(_) => HomeVariant::Welcome,
        };
        Self::with_variant(logo, variant)
    }

    fn with_variant(logo: AsciiLogoComponent, variant: HomeVariant) -> Self {
        let menu = Self::menu_for_variant(variant);
        let welcome_text = "Welcome! It looks like this is your first time here — let’s create a config to get you set up for work."
            .to_string();
        Self {
            logo,
            menu,
            variant,
            welcome_text,
        }
    }

    fn set_variant(&mut self, variant: HomeVariant) {
        self.variant = variant;
        self.menu = Self::menu_for_variant(variant);
    }

    fn menu_for_variant(variant: HomeVariant) -> Menu {
        match variant {
            HomeVariant::Welcome => Menu::new(vec![
                MenuItem::new("ok", "Ok"),
                MenuItem::new("quit", "Quit"),
            ]),
            HomeVariant::Default => Menu::new(vec![
                MenuItem::new("current_sprint", "Current Sprint").with_hint("c"),
                MenuItem::new("my_issues", "My issues").with_hint("i"),
                MenuItem::new("search_issues", "Search Issues").with_hint("s"),
                MenuItem::new("new_issue", "New Issue").with_hint("n"),
                MenuItem::new("refresh", "Refresh").with_hint("r"),
                MenuItem::new("profiles", "Profiles").with_hint("p"),
                MenuItem::new("quit", "Quit").with_hint("q"),
            ]),
        }
    }

    fn handle_confirm(&mut self) -> ScreenState {
        let Some(item) = self.menu.selected() else {
            return ScreenState::Stay;
        };

        match (self.variant, item.id) {
            (HomeVariant::Welcome, "ok") => ScreenState::SwitchTo(ScreenType::ProfileCreation),
            (HomeVariant::Welcome, "quit") => ScreenState::Quit,
            (HomeVariant::Default, "current_sprint") => {
                ScreenState::SwitchTo(ScreenType::CurrentSprint)
            }
            (HomeVariant::Default, "my_issues") => ScreenState::SwitchTo(ScreenType::MyIssues),
            (HomeVariant::Default, "search_issues") => {
                ScreenState::SwitchTo(ScreenType::SearchIssues)
            }
            (HomeVariant::Default, "new_issue") => ScreenState::SwitchTo(ScreenType::NewIssue),
            (HomeVariant::Default, "profiles") => ScreenState::SwitchTo(ScreenType::Profiles),
            (HomeVariant::Default, "refresh") => ScreenState::Refresh,
            (HomeVariant::Default, "quit") => ScreenState::Quit,
            _ => ScreenState::Stay,
        }
    }
}

impl Widget for HomeScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.logo.render(area, buf);
    }
}

impl Screen for HomeScreen {
    fn draw(&mut self, frame: &mut Frame) {
        match self.variant {
            HomeVariant::Welcome => {
                let screen_layout = Layout::vertical([
                    Constraint::Length(10),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Length(self.menu.height()),
                    Constraint::Fill(1),
                ])
                .split(frame.area());

                frame.render_widget(self.logo.clone(), screen_layout[1]);

                let content = Paragraph::new(Text::from(self.welcome_text.clone()))
                    .alignment(Alignment::Center)
                    .wrap(Wrap::default());
                frame.render_widget(content, screen_layout[2]);

                frame.render_widget(&self.menu, screen_layout[3]);
            }
            HomeVariant::Default => {
                let screen_layout = Layout::vertical([
                    Constraint::Length(10),
                    Constraint::Fill(1),
                    Constraint::Length(self.menu.height()),
                    Constraint::Fill(1),
                ])
                .split(frame.area());

                frame.render_widget(self.logo.clone(), screen_layout[1]);
                frame.render_widget(&self.menu, screen_layout[2]);
            }
        }
    }

    fn name(&self) -> &'static str {
        "Home Screen"
    }
}

impl KeyHandler for HomeScreen {
    fn handle_command(&mut self, command: Command) -> ScreenState {
        match command.action {
            ActionId::Refresh => ScreenState::Refresh,
            ActionId::MoveUp => {
                self.menu.move_up(command.repeat);
                ScreenState::Refresh
            }
            ActionId::MoveDown => {
                self.menu.move_down(command.repeat);
                ScreenState::Refresh
            }
            ActionId::Confirm => self.handle_confirm(),
            _ => ScreenState::Stay,
        }
    }
}
