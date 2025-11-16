//! Base Widget trait and input event definitions for ISSUN UI
//!
//! This module contains the core abstraction for all UI widgets,
//! independent of specific TUI backend libraries.

/// Base Widget trait for TUI components
pub trait Widget {
    /// Handle input event
    ///
    /// Returns `true` if the input was handled, `false` otherwise.
    fn handle_input(&mut self, _event: InputEvent) -> bool {
        false // Not handled by default
    }

    /// Get the name/type of this widget (for debugging)
    fn widget_type(&self) -> &'static str {
        "Widget"
    }
}

/// Input events abstracted from specific TUI libraries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// Move cursor up
    Up,
    /// Move cursor down
    Down,
    /// Move cursor left
    Left,
    /// Move cursor right
    Right,
    /// Confirm/Select current item
    Select,
    /// Cancel/Back
    Cancel,
    /// Tab key
    Tab,
    /// Character input
    Char(char),
    /// Function key
    Function(u8),
    /// Other/Unknown
    Other,
}

impl From<crossterm::event::KeyCode> for InputEvent {
    fn from(key: crossterm::event::KeyCode) -> Self {
        use crossterm::event::KeyCode;
        match key {
            KeyCode::Up | KeyCode::Char('k') => InputEvent::Up,
            KeyCode::Down | KeyCode::Char('j') => InputEvent::Down,
            KeyCode::Left | KeyCode::Char('h') => InputEvent::Left,
            KeyCode::Right | KeyCode::Char('l') => InputEvent::Right,
            KeyCode::Enter => InputEvent::Select,
            KeyCode::Esc | KeyCode::Char('q') => InputEvent::Cancel,
            KeyCode::Tab => InputEvent::Tab,
            KeyCode::Char(c) => InputEvent::Char(c),
            KeyCode::F(n) => InputEvent::Function(n),
            _ => InputEvent::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_event_conversion() {
        use crossterm::event::KeyCode;

        assert_eq!(InputEvent::from(KeyCode::Up), InputEvent::Up);
        assert_eq!(InputEvent::from(KeyCode::Enter), InputEvent::Select);
        assert_eq!(InputEvent::from(KeyCode::Char('k')), InputEvent::Up);
        assert_eq!(InputEvent::from(KeyCode::Char('a')), InputEvent::Char('a'));
    }
}
