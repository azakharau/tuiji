use color_eyre::eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, TerminalOptions, Viewport};
use tuiji::app::AppState;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init_with_options(TerminalOptions {
        viewport: Viewport::Fullscreen,
    });

    let app = AppState::default();

    let result = run(terminal, app);
    ratatui::restore();
    println!();
    result
}

fn run(mut terminal: DefaultTerminal, _app: AppState) -> Result<()> {
    terminal.clear()?;

    loop {
        let home_screen = tuiji::ui::screens::home::HomeScreen::default();
        terminal.draw(|frame| {
            home_screen.draw(frame);
        })?;
        if matches!(event::read()?, Event::Key(_)) {
            terminal.clear()?;
            break Ok(());
        }
    }
}
