use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    Redraw,
    TogglePause,
    SpeedUp,
    SlowDown,
    None,
}

/// Poll for keyboard input with a timeout. Returns the action to take.
pub fn poll_input(timeout: Duration) -> Action {
    if event::poll(timeout).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            return match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Action::Quit
                }
                KeyCode::Char('r') => Action::Redraw,
                KeyCode::Char('p') => Action::TogglePause,
                KeyCode::Char('f') | KeyCode::Right => Action::SpeedUp,
                KeyCode::Char('s') | KeyCode::Left => Action::SlowDown,
                _ => Action::None,
            };
        }
        if let Ok(Event::Resize(_, _)) = event::read() {
            return Action::Redraw;
        }
    }
    Action::None
}
