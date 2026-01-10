use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Paragraph, Widget, Wrap},
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
        context::RenderContext,
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
            AppConfigState::Loaded(cfg) => {
                if cfg.profiles.is_empty() {
                    HomeVariant::Welcome
                } else {
                    HomeVariant::Default
                }
            }
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

    fn menu_for_variant(variant: HomeVariant) -> Menu {
        match variant {
            HomeVariant::Welcome => Menu::new(vec![
                MenuItem::new("ok", "Ok"),
                MenuItem::new("quit", "Quit"),
            ]),
            HomeVariant::Default => Menu::new(vec![
                MenuItem::new("current_sprint", "Current Sprint").with_hint("c"),
                MenuItem::new("my_issues", "My issues").with_hint("i"),
                MenuItem::new("new_issue", "New Issue").with_hint("n"),
                MenuItem::new("boards", "Boards").with_hint("b"),
                MenuItem::new("settings", "Settings").with_hint(","),
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
            (HomeVariant::Default, "new_issue") => ScreenState::SwitchTo(ScreenType::NewIssue),
            (HomeVariant::Default, "boards") => ScreenState::SwitchTo(ScreenType::BoardSelection),
            (HomeVariant::Default, "settings") => ScreenState::SwitchTo(ScreenType::Settings),
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
    fn draw(&mut self, frame: &mut Frame, _context: &RenderContext) {
        let base_style = Style::default()
            .fg(_context.colors().text)
            .bg(_context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());

        self.menu.set_style(base_style);
        self.menu.set_highlight_style(
            Style::default()
                .fg(_context.colors().text)
                .bg(_context.colors().selection)
                .add_modifier(Modifier::BOLD),
        );

        let logo_style = Style::default().fg(_context.colors().logo);
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

                frame.render_widget(self.logo.clone().with_style(logo_style), screen_layout[1]);

                let content = Paragraph::new(Text::from(self.welcome_text.clone()))
                    .style(base_style)
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

                frame.render_widget(self.logo.clone().with_style(logo_style), screen_layout[1]);
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
            ActionId::MoveTop => {
                self.menu.move_top();
                ScreenState::Refresh
            }
            ActionId::MoveBottom => {
                self.menu.move_bottom();
                ScreenState::Refresh
            }
            ActionId::Confirm => self.handle_confirm(),
            _ => ScreenState::Stay,
        }
    }
}
