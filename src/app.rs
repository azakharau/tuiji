use color_eyre::eyre::Result;
use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use crate::{
    app::state::ScreenType,
    ui::screens::{Screen, ScreenAction, home::HomeScreen},
};

pub mod state;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub mode: state::Mode,
    pub current_screen: state::ScreenType,
}

struct CachedScreens {
    home_screen: Option<HomeScreen>,
}

impl CachedScreens {
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
                unreachable!("Screen not implemented");
            }
        }
    }
}

pub struct App {
    pub terminal: DefaultTerminal,
    pub state: AppState,
    screen: CachedScreens,
}

impl App {
    pub fn new(terminal: DefaultTerminal, state: AppState) -> Self {
        let screen = CachedScreens { home_screen: None };
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
            if let Event::Key(key) = event::read()? {
                match screen.handle_key_event(key) {
                    ScreenAction::Quit => {
                        self.terminal.clear()?;
                        break Ok(());
                    }
                    ScreenAction::SwitchTo(new_screen) => {
                        self.state.current_screen = new_screen;
                    }
                    ScreenAction::Refresh => {
                        // No-op, just redraw
                    }
                    ScreenAction::Stay => {
                        // No-op, just redraw
                    }
                }
            }
        }
    }
}
