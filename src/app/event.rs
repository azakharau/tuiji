use crossterm::event::KeyEvent;

pub enum EventKind {
    Input(KeyEvent),
    Tick,
}
