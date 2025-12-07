pub mod state;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub mode: state::Mode,
    pub screen: state::ScreenType,
}

impl AppState {
    pub fn handle_mode_change(&mut self, new_mode: state::Mode) {
        self.mode = new_mode;
    }
}
