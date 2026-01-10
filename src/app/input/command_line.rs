use super::TextInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandLineAction {
    Write,
    WriteQuit,
    Quit,
    QuitAll,
    WriteQuitAll,
    Sync(SyncAction),
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    Pull,
    Push,
    SwitchOffline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandLineOutcome {
    Updated,
    Submitted(Option<CommandLineAction>),
    Cancelled,
    Noop,
}

#[derive(Debug, Default, Clone)]
pub struct CommandLineState {
    buffer: String,
    active: bool,
}

impl CommandLineState {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            active: false,
        }
    }

    pub fn start(&mut self) {
        self.buffer.clear();
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.buffer.clear();
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn buffer(&self) -> Option<&str> {
        if self.active {
            Some(self.buffer.as_str())
        } else {
            None
        }
    }

    pub fn handle_event(&mut self, event: TextInput) -> CommandLineOutcome {
        if !self.active {
            return CommandLineOutcome::Noop;
        }
        match event {
            TextInput::Char(ch) => {
                self.buffer.push(ch);
                CommandLineOutcome::Updated
            }
            TextInput::Backspace | TextInput::Delete => {
                self.buffer.pop();
                CommandLineOutcome::Updated
            }
            TextInput::Enter => {
                let cmd = self.buffer.trim().to_string();
                if cmd.is_empty() {
                    self.buffer.clear();
                    self.active = false;
                    return CommandLineOutcome::Submitted(None);
                }
                let action = parse_command(&cmd);
                if matches!(action, Some(CommandLineAction::Invalid)) {
                    return CommandLineOutcome::Submitted(action);
                }
                self.buffer.clear();
                self.active = false;
                CommandLineOutcome::Submitted(action)
            }
            TextInput::Esc => {
                self.buffer.clear();
                self.active = false;
                CommandLineOutcome::Cancelled
            }
            TextInput::Tab => CommandLineOutcome::Noop,
        }
    }
}

fn parse_command(cmd: &str) -> Option<CommandLineAction> {
    let cmd = cmd.trim();
    match cmd {
        "w" => Some(CommandLineAction::Write),
        "wq" | "x" => Some(CommandLineAction::WriteQuit),
        "q" | "q!" => Some(CommandLineAction::Quit),
        "qa" | "qall" | "quitall" | "qa!" | "qall!" => Some(CommandLineAction::QuitAll),
        "wqa" | "wqall" | "xall" | "xa" => Some(CommandLineAction::WriteQuitAll),
        _ => {
            let mut parts = cmd.split_whitespace();
            let Some(head) = parts.next() else {
                return Some(CommandLineAction::Invalid);
            };
            if head != "sync" {
                return Some(CommandLineAction::Invalid);
            }
            match parts.next() {
                None => Some(CommandLineAction::Sync(SyncAction::Pull)),
                Some("pull") => Some(CommandLineAction::Sync(SyncAction::Pull)),
                Some("push") => Some(CommandLineAction::Sync(SyncAction::Push)),
                Some("offline") | Some("cache") => {
                    Some(CommandLineAction::Sync(SyncAction::SwitchOffline))
                }
                _ => Some(CommandLineAction::Invalid),
            }
        }
    }
}
