use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyEvent};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    app::{
        event::{AppEvent, WorkerMessage},
        key_handlers::{global_action_hints, parse_command, Command, InputState},
        state::ScreenType,
    },
    config::AppConfig,
    ui::screens::{Screen, ScreenState, current_sprint, current_sprint::CurrentSprintScreen, home::HomeScreen},
};

pub mod event;
pub mod key_handlers;
pub mod state;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub mode: state::Mode,
    pub current_screen: state::ScreenType,
}

struct CachedScreens {
    home_screen: Option<HomeScreen>,
    current_sprint_screen: Option<CurrentSprintScreen>,
}

impl CachedScreens {
    pub fn active_mut(&mut self, _cfg: &AppConfig, state: &AppState) -> &mut dyn Screen {
        match state.current_screen {
            ScreenType::Home => {
                if self.home_screen.is_none() {
                    self.home_screen = Some(HomeScreen::default());
                }
                // SAFE: We just ensured it's Some above
                self.home_screen.as_mut().unwrap()
            }
            ScreenType::CurrentSprint => self
                .current_sprint_screen
                .as_mut()
                .expect("Current sprint screen not loaded"),
            _ => {
                panic!("Screen {:?} not implemented yet", state.current_screen);
            }
        }
    }
}

enum ActionOutcome {
    Continue { render: bool },
    Quit,
}

pub struct App {
    pub terminal: DefaultTerminal,
    pub state: AppState,
    screens: CachedScreens,
    cfg: AppConfig,
    input_state: InputState,
}

impl App {
    pub fn new(terminal: DefaultTerminal, state: AppState) -> Self {
        let screen = CachedScreens {
            home_screen: None,
            current_sprint_screen: None,
        };
        let config = AppConfig::load().unwrap();
        Self {
            terminal,
            state,
            screens: screen,
            cfg: config,
            input_state: InputState::default(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.terminal.clear()?;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let _input = spawn_input_listener(tx.clone());
        // let _tick = spawn_tick(tx.clone(), Duration::from_millis(250));
        // let _cache_worker = spawn_worker(tx.clone());
        // let _notification_worker = spawn_notifications(tx.clone());

        // Initial paint.
        self.render().await?;

        while let Some(event) = rx.recv().await {
            let mut render_requested = false;

            match event {
                AppEvent::Input(key) => {
                    // Ensure we preload any heavy screens before handling input to avoid blocking in async context.
                    self.ensure_screen_ready().await?;
                    let action = self.handle_input(key)?;
                    match self.apply_action(action)? {
                        ActionOutcome::Continue { render } => render_requested |= render,
                        ActionOutcome::Quit => {
                            break;
                        }
                    }
                }
                AppEvent::Tick => {
                    render_requested = true;
                }
                AppEvent::Worker(msg) => {
                    render_requested |= self.handle_worker(msg)?;
                }
            }

            if render_requested {
                self.render().await?;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) -> Result<ScreenState> {
        let cmd = parse_command(key, self.state.mode.clone(), &self.cfg.key_bindings, &mut self.input_state);
        match cmd {
            Command::Quit => Ok(ScreenState::Quit),
            Command::Refresh => Ok(ScreenState::Refresh),
            Command::SwitchTo(screen) => Ok(ScreenState::SwitchTo(screen)),
            Command::Motion(_) | Command::Noop | Command::Unhandled(_) => {
                let screen = self.screens.active_mut(&self.cfg, &self.state);
                Ok(screen.handle_command(cmd))
            }
        }
    }

    fn apply_action(&mut self, action: ScreenState) -> Result<ActionOutcome> {
        match action {
            ScreenState::Quit => {
                self.terminal.clear()?;
                Ok(ActionOutcome::Quit)
            }
            ScreenState::SwitchTo(new_screen) => {
                self.state.current_screen = new_screen;
                self.terminal.clear()?;
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::Refresh => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::Stay => Ok(ActionOutcome::Continue { render: false }),
        }
    }

    fn handle_worker(&mut self, _msg: WorkerMessage) -> Result<bool> {
        // Placeholder: update state based on worker notifications when added.
        Ok(true)
    }

    async fn render(&mut self) -> Result<()> {
        self.ensure_screen_ready().await?;
        let screen_type = self.state.current_screen.clone();
        let screen = self.screens.active_mut(&self.cfg, &self.state);
        let hints = match screen_type {
            ScreenType::CurrentSprint => {
                current_sprint::CurrentSprintScreen::action_hints(&self.cfg.key_bindings)
            }
            _ => global_action_hints(&self.cfg.key_bindings),
        };
        screen.set_action_hints(hints);
        self.terminal.draw(|frame| {
            screen.draw(frame);
        })?;
        Ok(())
    }

    async fn ensure_screen_ready(&mut self) -> Result<()> {
        if matches!(self.state.current_screen, ScreenType::CurrentSprint)
            && self.screens.current_sprint_screen.is_none()
        {
            let cfg = self.cfg.clone();
            let mode = self.state.mode.clone();
            let screen = CurrentSprintScreen::new(&cfg, mode).await?;
            self.screens.current_sprint_screen = Some(screen);
        }
        Ok(())
    }
}

fn spawn_input_listener(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(event) = reader.next().await {
            match event {
                Ok(Event::Key(key)) => {
                    let _ = tx.send(AppEvent::Input(key));
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("event stream error: {err}");
                    break;
                }
            }
        }
    })
}

// fn spawn_tick(tx: UnboundedSender<AppEvent>, period: Duration) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         let mut ticker = time::interval(period);
//         loop {
//             ticker.tick().await;
//             if tx.send(AppEvent::Tick).is_err() {
//                 break;
//             }
//         }
//     })
// }
//
// fn spawn_worker(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         // Placeholder for cache/notification workers; send messages when ready.
//         // let _ = tx.send(AppEvent::Worker(WorkerMessage::JiraUpdated));
//         let _ = tx;
//     })
// }
//
// fn spawn_notifications(tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         let mut ticker = time::interval(NOTIFY_POLL_INTERVAL);
//         loop {
//             ticker.tick().await;
//             // placeholder: poll notification source, then:
//             // if tx.send(AppEvent::Worker(WorkerMessage::Notification(payload))).is_err() { break; }
//             if tx.is_closed() {
//                 break;
//             }
//         }
//     })
// }

// Legacy stub retained to avoid unused warnings when adding more global mappings later.
#[allow(dead_code)]
fn map_global(_key: &KeyEvent, _bindings: &crate::config::KeyBindings) -> Option<ScreenState> {
    None
}
