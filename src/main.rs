use color_eyre::eyre::Result;
use ratatui::{TerminalOptions, Viewport};
use tuiji::app::{App, AppState};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init_with_options(TerminalOptions {
        viewport: Viewport::Fullscreen,
    });

    let state = AppState::default();
    let mut app = App::new(terminal, state)?;

    let result = app.run().await;
    ratatui::restore();
    println!();
    result
}
