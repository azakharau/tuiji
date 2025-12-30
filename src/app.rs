use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyEvent};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear},
};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    app::{
        event::{AppEvent, WorkerMessage},
        input::{
            CommandLineAction, CommandLineOutcome, CommandLineState, InputEvent, InputParser,
            is_question_mark,
        },
        input::overlay::command_line_area,
        key_handlers::{
            ActionId, Command, action_hints, binding_hints_for_prefix, binding_hints_for_screen,
        },
        state::{Mode, ScreenType},
    },
    config::{AppConfig, AppConfigState},
    ui::{
        components::{
            command_line::CommandLine,
            logo::AsciiLogoComponent,
            which_key_popup::WhichKeyPopup,
        },
        screens::{
            CommandLineCommand, Screen, ScreenState, current_sprint::CurrentSprintScreen,
            home::HomeScreen, profile_creation::ProfileCreationScreen,
        },
    },
};

pub mod event;
pub mod input;
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
    profile_creation: Option<ProfileCreationScreen>,
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
                    let logo = AsciiLogoComponent::default();
                    self.home_screen = Some(HomeScreen::new(logo, cfg_state));
                }
                Ok(self.home_screen.as_mut().expect("Home screen not loaded"))
            }
            ScreenType::CurrentSprint => {
                if self.current_sprint_screen.is_none() {
                    let mode = state.mode;
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
                if self.profile_creation.is_none() {
                    self.profile_creation = Some(ProfileCreationScreen::new());
                }
                Ok(self
                    .profile_creation
                    .as_mut()
                    .expect("Profile screen not loaded"))
            }
            ScreenType::ProfileCreation => {
                if self.profile_creation.is_none() {
                    self.profile_creation = Some(ProfileCreationScreen::new());
                }
                Ok(self
                    .profile_creation
                    .as_mut()
                    .expect("Profile creation screen not loaded"))
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
    input: InputParser,
    screen_stack: Vec<ScreenType>,
    command_line: CommandLineState,
    show_hints: bool,
}

impl App {
    pub fn new(terminal: DefaultTerminal, state: AppState) -> Result<Self> {
        let screen = CachedScreens {
            home_screen: None,
            current_sprint_screen: None,
            profile_creation: None,
        };
        let config = AppConfig::load_state();
        Ok(Self {
            terminal,
            state,
            screens: screen,
            cfg_state: config,
            input: InputParser::default(),
            screen_stack: Vec::new(),
            command_line: CommandLineState::new(),
            show_hints: false,
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
        if self.show_hints && !is_question_mark(&key) {
            self.show_hints = false;
        }
        let had_prefix = self.input.pending_prefix().is_some();
        let event = self
            .input
            .parse(key, self.state.mode, self.state.current_screen);
        let has_prefix = self.input.pending_prefix().is_some();

        let Some(event) = event else {
            if had_prefix != has_prefix || has_prefix {
                return Ok(ScreenState::Refresh);
            }
            return Ok(ScreenState::Stay);
        };

        self.handle_input_event(event).await
    }

    async fn handle_input_event(&mut self, event: InputEvent) -> Result<ScreenState> {
        match event {
            InputEvent::Action(cmd) => self.handle_action_command(cmd).await,
            InputEvent::ModeSwitch(mode) => {
                if mode == Mode::Command && !self.command_mode_allowed() {
                    return Ok(ScreenState::Stay);
                }
                self.set_mode(mode);
                Ok(ScreenState::Refresh)
            }
            InputEvent::ToggleHints => {
                self.show_hints = !self.show_hints;
                Ok(ScreenState::Refresh)
            }
            InputEvent::Text(ch) => {
                match self.state.mode {
                    Mode::Command => self.handle_command_line_event(InputEvent::Text(ch)).await,
                    Mode::Insert => {
                        self.forward_raw_input(crossterm::event::KeyCode::Char(ch))
                            .await
                    }
                    _ => Ok(ScreenState::Stay),
                }
            }
            InputEvent::Backspace => {
                match self.state.mode {
                    Mode::Command => self.handle_command_line_event(InputEvent::Backspace).await,
                    Mode::Insert => {
                        self.forward_raw_input(crossterm::event::KeyCode::Backspace)
                            .await
                    }
                    _ => Ok(ScreenState::Stay),
                }
            }
            InputEvent::Delete => {
                match self.state.mode {
                    Mode::Command => self.handle_command_line_event(InputEvent::Delete).await,
                    Mode::Insert => {
                        self.forward_raw_input(crossterm::event::KeyCode::Delete)
                            .await
                    }
                    _ => Ok(ScreenState::Stay),
                }
            }
            InputEvent::Enter => {
                match self.state.mode {
                    Mode::Command => self.handle_command_line_event(InputEvent::Enter).await,
                    Mode::Insert => {
                        self.forward_raw_input(crossterm::event::KeyCode::Enter)
                            .await
                    }
                    _ => Ok(ScreenState::Stay),
                }
            }
            InputEvent::Tab => {
                match self.state.mode {
                    Mode::Command => self.handle_command_line_event(InputEvent::Tab).await,
                    Mode::Insert => {
                        self.forward_raw_input(crossterm::event::KeyCode::Tab)
                            .await
                    }
                    _ => Ok(ScreenState::Stay),
                }
            }
            InputEvent::Esc => {
                match self.state.mode {
                    Mode::Command => self.handle_command_line_event(InputEvent::Esc).await,
                    Mode::Insert | Mode::Visual => {
                        self.set_mode(Mode::Normal);
                        Ok(ScreenState::Refresh)
                    }
                    _ => Ok(ScreenState::Stay),
                }
            }
        }
    }

    async fn handle_action_command(&mut self, cmd: Command) -> Result<ScreenState> {
        if let ActionId::EnterInsert(_) = cmd.action {
            self.set_mode(Mode::Insert);
        }
        // Global navigation/actions handled here; the rest go to the active screen.
        if let Some(nav) = self.map_global_action(&cmd) {
            return Ok(nav);
        }

        let screen = self
            .screens
            .active_mut(&self.cfg_state, &self.state)
            .await?;
        Ok(screen.handle_command(cmd))
    }

    async fn forward_raw_input(&mut self, code: crossterm::event::KeyCode) -> Result<ScreenState> {
        let screen = self
            .screens
            .active_mut(&self.cfg_state, &self.state)
            .await?;
        Ok(screen.handle_command(Command {
            action: ActionId::RawInput(code),
            repeat: 1,
        }))
    }

    fn apply_action(&mut self, action: ScreenState) -> Result<ActionOutcome> {
        match action {
            ScreenState::Quit => {
                self.terminal.clear()?;
                Ok(ActionOutcome::Quit)
            }
            ScreenState::SwitchTo(new_screen) => {
                if new_screen == ScreenType::Home {
                    self.screen_stack.clear();
                } else if new_screen != self.state.current_screen {
                    self.screen_stack.push(self.state.current_screen);
                }
                self.state.current_screen = new_screen;
                if self.state.mode == Mode::Command && !self.command_mode_allowed() {
                    self.set_mode(Mode::Normal);
                }
                self.terminal.clear()?;
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::Refresh => Ok(ActionOutcome::Continue { render: true }),
            ScreenState::Stay => Ok(ActionOutcome::Continue { render: false }),
            ScreenState::SaveConfig(cfg) => {
                self.save_config(cfg)?;
                Ok(ActionOutcome::Continue { render: true })
            }
            ScreenState::SaveAndClose(cfg) => {
                self.save_config(cfg)?;
                self.close_screen()
            }
            ScreenState::Close => self.close_screen(),
        }
    }

    fn handle_worker(&mut self, _msg: WorkerMessage) -> Result<bool> {
        // Placeholder: update state based on worker notifications when added.
        Ok(true)
    }

    fn map_global_action(&mut self, cmd: &Command) -> Option<ScreenState> {
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
        let screen_type = self.state.current_screen;
        let screen = self
            .screens
            .active_mut(&self.cfg_state, &self.state)
            .await?;
        let hints = action_hints(screen_type);
        screen.set_action_hints(hints);
        screen.set_mode(self.state.mode);
        self.terminal.draw(|frame| {
            screen.draw(frame);
            if self.show_hints {
                let hints = binding_hints_for_screen(screen_type);
                let popup = WhichKeyPopup::new("Key Hints".to_string(), hints);
                frame.render_widget(&popup, frame.area());
            } else if let Some(prefix) = self.input.pending_prefix() {
                let hints = binding_hints_for_prefix(screen_type, &prefix);
                let popup = WhichKeyPopup::new(prefix, hints);
                frame.render_widget(&popup, frame.area());
            }
            if let Some(buffer) = self.command_line.buffer() {
                let area = command_line_area(frame.area());
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(Line::from("Command").centered())
                    .title_style(Style::default().fg(Color::Yellow));
                frame.render_widget(Clear, area);
                frame.render_widget(&block, area);
                frame.render_widget(CommandLine::new(buffer), block.inner(area));
            }
        })?;
        Ok(())
    }

    fn set_mode(&mut self, mode: Mode) {
        self.state.mode = mode;
        self.input.clear_pending();
        if mode == Mode::Command {
            if !self.command_mode_allowed() {
                self.state.mode = Mode::Normal;
                self.command_line.stop();
                return;
            }
            self.command_line.start();
        } else {
            self.command_line.stop();
        }
    }

    async fn handle_command_line_event(&mut self, event: InputEvent) -> Result<ScreenState> {
        match self.command_line.handle_event(event) {
            CommandLineOutcome::Updated => Ok(ScreenState::Refresh),
            CommandLineOutcome::Cancelled => {
                self.set_mode(Mode::Normal);
                Ok(ScreenState::Refresh)
            }
            CommandLineOutcome::Submitted(action) => {
                self.set_mode(Mode::Normal);
                if let Some(action) = action {
                    self.handle_command_line_action(action).await
                } else {
                    Ok(ScreenState::Stay)
                }
            }
            CommandLineOutcome::Noop => Ok(ScreenState::Stay),
        }
    }

    async fn handle_command_line_action(
        &mut self,
        action: CommandLineAction,
    ) -> Result<ScreenState> {
        match action {
            CommandLineAction::Write => {
                let screen = self
                    .screens
                    .active_mut(&self.cfg_state, &self.state)
                    .await?;
                Ok(screen.handle_command_line(CommandLineCommand::Write))
            }
            CommandLineAction::WriteQuit => {
                let screen = self
                    .screens
                    .active_mut(&self.cfg_state, &self.state)
                    .await?;
                Ok(screen.handle_command_line(CommandLineCommand::WriteQuit))
            }
            CommandLineAction::WriteQuitAll => {
                let screen = self
                    .screens
                    .active_mut(&self.cfg_state, &self.state)
                    .await?;
                let res = screen.handle_command_line(CommandLineCommand::Write);
                if let ScreenState::SaveConfig(cfg) | ScreenState::SaveAndClose(cfg) = res {
                    self.save_config(cfg)?;
                }
                Ok(ScreenState::Quit)
            }
            CommandLineAction::Quit => Ok(ScreenState::Close),
            CommandLineAction::QuitAll => Ok(ScreenState::Quit),
        }
    }

    fn save_config(&mut self, cfg: AppConfig) -> Result<()> {
        cfg.save()?;
        self.cfg_state = AppConfigState::Loaded(cfg);
        self.screens.home_screen = None;
        self.screens.current_sprint_screen = None;
        Ok(())
    }

    fn command_mode_allowed(&self) -> bool {
        !matches!(self.state.current_screen, ScreenType::Home)
    }

    fn close_screen(&mut self) -> Result<ActionOutcome> {
        if let Some(prev) = self.screen_stack.pop() {
            self.state.current_screen = prev;
            self.terminal.clear()?;
            Ok(ActionOutcome::Continue { render: true })
        } else {
            self.terminal.clear()?;
            Ok(ActionOutcome::Quit)
        }
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
