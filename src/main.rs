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

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     use tuiji::client::jira::JiraClient;
//     use tuiji::config::AppConfig;
//     let cfg = AppConfig::load()?;
//     let jira = JiraClient::new(&cfg.jira.base_url, &cfg.jira.username, &cfg.jira.api_token);
//     let res = jira.get_current_sprint_issues(175_u64)?;
//     dbg!(res);
//
//     Ok(())
// }
