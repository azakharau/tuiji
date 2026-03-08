use crate::app::{
    AppState,
    input::{CommandLineState, InputParser},
    state::Mode,
};

pub struct StateMutationPolicy;

impl StateMutationPolicy {
    pub fn apply_mode(
        state: &mut AppState,
        input: &mut InputParser,
        command_line: &mut CommandLineState,
        mode: Mode,
        command_mode_allowed: bool,
    ) {
        state.mode = mode;
        input.clear_pending();
        if mode == Mode::Command {
            if !command_mode_allowed {
                state.mode = Mode::Normal;
                command_line.stop();
                return;
            }
            command_line.start();
        } else {
            command_line.stop();
        }
    }
}
