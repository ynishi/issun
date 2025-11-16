//! Input polling utilities for game loops
//!
//! Provides timeout-based input polling for non-blocking game loops.

use crate::ui::core::widget::InputEvent;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

/// Poll for input events with timeout
///
/// Returns `InputEvent::Other` if timeout expires without input.
/// Only processes key press events (ignores key repeat/release).
///
/// # Arguments
///
/// * `timeout` - Maximum time to wait for input
///
/// # Example
///
/// ```ignore
/// use issun::ui::input::poll_input;
/// use std::time::Duration;
///
/// loop {
///     let event = poll_input(Duration::from_millis(100))?;
///     match event {
///         InputEvent::Quit => break,
///         InputEvent::Up => player.move_up(),
///         InputEvent::Other => { /* timeout, continue game loop */ }
///         _ => {}
///     }
/// }
/// ```
pub fn poll_input(timeout: Duration) -> std::io::Result<InputEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key_event) = event::read()? {
            // Only process key press events (ignore repeat/release)
            if key_event.kind == KeyEventKind::Press {
                return Ok(InputEvent::from(key_event.code));
            }
        }
    }
    Ok(InputEvent::Other)
}

/// Poll for raw key code with timeout
///
/// Similar to `poll_input()` but returns the raw KeyCode instead of InputEvent.
/// Useful when you need direct access to key codes not mapped in InputEvent.
///
/// # Example
///
/// ```ignore
/// use issun::ui::input::poll_key;
/// use crossterm::event::KeyCode;
/// use std::time::Duration;
///
/// if let Some(key) = poll_key(Duration::from_millis(100))? {
///     match key {
///         KeyCode::Char('1') => select_option(1),
///         KeyCode::Char('2') => select_option(2),
///         _ => {}
///     }
/// }
/// ```
pub fn poll_key(timeout: Duration) -> std::io::Result<Option<KeyCode>> {
    if event::poll(timeout)? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                return Ok(Some(key_event.code));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_input_timeout() {
        // This test just verifies the function compiles and returns Ok
        // Real testing requires terminal interaction
        let result = poll_input(Duration::from_millis(1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_poll_key_timeout() {
        let result = poll_key(Duration::from_millis(1));
        assert!(result.is_ok());
    }
}
