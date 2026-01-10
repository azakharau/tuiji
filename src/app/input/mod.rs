pub mod command;
pub mod command_line;
pub mod overlay;
pub mod parser;

pub use command::{CommandResolver, InputCommand};
pub use command_line::{CommandLineAction, CommandLineOutcome, CommandLineState, SyncAction};
pub use parser::{InputParser, ParsedInput, TextInput};

use crossterm::event::KeyEvent;

pub fn is_question_mark(key: &KeyEvent) -> bool {
    matches!(key.code, crossterm::event::KeyCode::Char('?'))
}
