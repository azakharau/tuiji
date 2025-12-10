use color_eyre::eyre::Result;
use crossterm::event::{self as term_event, Event};
use ratatui::DefaultTerminal;

use crate::{
    app::state::ScreenType,
    ui::screens::{Screen, ScreenState, current_sprint::CurrentSprintScreen, home::HomeScreen},
};

pub mod datasets;
pub mod event;
pub mod key_handlers;
pub mod state;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub mode: state::Mode,
    pub current_screen: state::ScreenType,
}

struct CachedScreens<'a> {
    home_screen: Option<HomeScreen>,
    current_sprint_screen: Option<CurrentSprintScreen<'a>>,
}

impl<'a> CachedScreens<'a> {
    pub fn active_mut(&mut self, which: &state::ScreenType) -> &mut dyn Screen {
        match which {
            ScreenType::Home => {
                if self.home_screen.is_none() {
                    self.home_screen = Some(HomeScreen::default());
                }
                // SAFE: We just ensured it's Some above
                self.home_screen.as_mut().unwrap()
            }
            _ => {
                if self.current_sprint_screen.is_none() {
                    self.current_sprint_screen = Some(CurrentSprintScreen::default());
                }
                // SAFE: We just ensured it's Some above
                self.current_sprint_screen.as_mut().unwrap()
            }
        }
    }
}

pub struct App<'a> {
    pub terminal: DefaultTerminal,
    pub state: AppState,
    screen: CachedScreens<'a>,
}

impl<'a> App<'a> {
    pub fn new(terminal: DefaultTerminal, state: AppState) -> Self {
        let screen = CachedScreens {
            home_screen: None,
            current_sprint_screen: None,
        };
        Self {
            terminal,
            state,
            screen,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.terminal.clear()?;

        loop {
            let screen = self.screen.active_mut(&self.state.current_screen);
            self.terminal.draw(|frame| {
                screen.draw(frame);
            })?;
            if let Event::Key(key) = term_event::read()? {
                match screen.handle_key_event(key) {
                    ScreenState::Quit => {
                        self.terminal.clear()?;
                        break Ok(());
                    }
                    ScreenState::SwitchTo(new_screen) => {
                        self.state.current_screen = new_screen;
                    }
                    ScreenState::Refresh => {
                        // No-op, just redraw
                    }
                    ScreenState::Stay => {
                        // No-op, just redraw
                    }
                    _ => {
                        // TODO: Handle other actions
                    }
                }
            }
        }
    }
}
