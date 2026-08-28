use color_eyre::eyre::Result;
use tuiji::app::{App, AppState};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::try_init()?;

    let state = AppState::default();
    let mut app = App::new(terminal, state)?;

    let result = app.run().await;
    let restored = ratatui::try_restore().map_err(color_eyre::Report::from);
    println!();
    result.and(restored)
}
