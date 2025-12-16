use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyEvent};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    app::{
        event::{AppEvent, WorkerMessage},
        key_handlers::{action_hints, parse_command, ActionId, Command, InputState},
        state::ScreenType,
    },
    config::{AppConfig, AppConfigState},
    ui::screens::{
        Screen, ScreenState, current_sprint::CurrentSprintScreen, home::HomeScreen,
        profile_picker::ProfileScreen,
    },
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
    profile_screen: Option<ProfileScreen>,
}

impl CachedScreens {
    pub async fn active_mut(
        &mut self,
        cfg_state: &AppConfigState,
        state: &AppState,
    ) -> Result<&mut dyn Screen> {
        let cfg = match cfg_state {
            AppConfigState::Loaded(c) => Some(c),
            AppConfigState::Missing(_) => None,
        };
        match state.current_screen {
            ScreenType::Home => {
                if self.home_screen.is_none() {
                    self.home_screen = Some(HomeScreen::default());
                }
                Ok(self.home_screen.as_mut().expect("Home screen not loaded"))
            }
            ScreenType::CurrentSprint => {
                if self.current_sprint_screen.is_none() {
                    let mode = state.mode.clone();
                    let cfg = cfg.ok_or_else(|| {
                        color_eyre::eyre::eyre!("Config missing: cannot open Current Sprint screen")
                    })?;
                    let screen = CurrentSprintScreen::new(cfg, mode).await?;
                    self.current_sprint_screen = Some(screen);
                }
                Ok(self
                    .current_sprint_screen
                    .as_mut()
                    .expect("Current sprint screen not loaded"))
            }
            ScreenType::Profiles => {
                if self.profile_screen.is_none() {
                    let cfg = cfg.ok_or_else(|| {
                        color_eyre::eyre::eyre!("Config missing: cannot open Profiles screen")
                    })?;
                    self.profile_screen = Some(ProfileScreen::new(cfg.clone()));
                }
                Ok(self
                    .profile_screen
                    .as_mut()
                    .expect("Profile screen not loaded"))
            }
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
    cfg_state: AppConfigState,
    input_state: InputState,
}

impl App {
    pub fn new(terminal: DefaultTerminal, state: AppState) -> Result<Self> {
        let screen = CachedScreens {
            home_screen: None,
            current_sprint_screen: None,
            profile_screen: None,
        };
        let config = AppConfig::load_state();
        Ok(Self {
            terminal,
            state,
            screens: screen,
            cfg_state: config,
            input_state: InputState::default(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.terminal.clear()?;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let _input = spawn_input_listener(tx.clone());
        // let _tick = spawn_tick(tx.clone(), Duration::from_millis(250));
        // let _cache_worker = spawn_worker(tx.clone());
        // let _notification_worker = spawn_notifications(tx.clone());

        // Initial paint.
        self.init_start_screen();
        self.render().await?;

        while let Some(event) = rx.recv().await {
            let mut render_requested = false;

            match event {
                AppEvent::Input(key) => {
                    let action = self.handle_input(key).await?;
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

    async fn handle_input(&mut self, key: KeyEvent) -> Result<ScreenState> {
        let maybe_cmd = parse_command(
            key,
            self.state.mode.clone(),
            &mut self.input_state,
            self.state.current_screen.clone(),
        );

        let Some(cmd) = maybe_cmd else {
            return Ok(ScreenState::Stay);
        };

        // Global navigation/actions handled here; the rest go to the active screen.
        if let Some(nav) = self.map_global_action(&cmd) {
            return Ok(nav);
        }

        let screen = self.screens.active_mut(&self.cfg_state, &self.state).await?;
        Ok(screen.handle_command(cmd))
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

    fn map_global_action(&self, cmd: &Command) -> Option<ScreenState> {
        match cmd.action {
            ActionId::Quit => Some(ScreenState::Quit),
            ActionId::Refresh => Some(ScreenState::Refresh),
            ActionId::GoHome => Some(ScreenState::SwitchTo(ScreenType::Home)),
            ActionId::OpenCurrentSprint => Some(ScreenState::SwitchTo(ScreenType::CurrentSprint)),
            ActionId::OpenProfiles => Some(ScreenState::SwitchTo(ScreenType::Profiles)),
            ActionId::OpenMyIssues => Some(ScreenState::SwitchTo(ScreenType::MyIssues)),
            ActionId::OpenSearchIssues => Some(ScreenState::SwitchTo(ScreenType::SearchIssues)),
            ActionId::OpenNewIssue => Some(ScreenState::SwitchTo(ScreenType::NewIssue)),
            _ => None,
        }
    }

    async fn render(&mut self) -> Result<()> {
        let screen_type = self.state.current_screen.clone();
        let screen = self.screens.active_mut(&self.cfg_state, &self.state).await?;
        let hints = action_hints(screen_type.clone());
        screen.set_action_hints(hints);
        self.terminal.draw(|frame| {
            screen.draw(frame);
        })?;
        Ok(())
    }

    fn init_start_screen(&mut self) {
        match self.cfg_state {
            AppConfigState::Loaded(_) => {
                self.state.current_screen = ScreenType::Home;
            }
            AppConfigState::Missing(_) => {
                // Placeholder: could route to Welcome screen later.
                self.state.current_screen = ScreenType::Home;
            }
        }
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
