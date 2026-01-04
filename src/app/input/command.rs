use crate::app::{
    key_handlers::{Command, KeyBindings, is_motion_action},
    state::ScreenType,
};

use super::{ParsedInput, TextInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputCommand {
    Action(Command),
    ModeSwitch(crate::app::state::Mode),
    ToggleHints,
    Text(TextInput),
}

pub struct CommandResolver<'a> {
    bindings: &'a KeyBindings,
}

impl<'a> CommandResolver<'a> {
    pub fn new(bindings: &'a KeyBindings) -> Self {
        Self { bindings }
    }

    pub fn resolve(&self, input: ParsedInput, screen: ScreenType) -> Option<InputCommand> {
        match input {
            ParsedInput::Binding { binding, repeat } => {
                let action = self.bindings.action_for_binding(screen, &binding)?;
                let repeat = if is_motion_action(action) { repeat } else { 1 };
                Some(InputCommand::Action(Command { action, repeat }))
            }
            ParsedInput::ModeSwitch(mode) => Some(InputCommand::ModeSwitch(mode)),
            ParsedInput::ToggleHints => Some(InputCommand::ToggleHints),
            ParsedInput::Text(text) => Some(InputCommand::Text(text)),
        }
    }
}
