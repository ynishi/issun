//! Terminal User Interface wrapper for ISSUN
//!
//! Provides a simple abstraction for terminal initialization and cleanup.

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

/// Terminal User Interface wrapper
///
/// Handles terminal initialization and cleanup automatically.
///
/// # Example
///
/// ```ignore
/// use issun::ui::Tui;
///
/// fn main() -> std::io::Result<()> {
///     let mut tui = Tui::new()?;
///
///     loop {
///         tui.terminal().draw(|f| {
///             // Render UI
///         })?;
///
///         // Handle input...
///         // Update game state...
///     }
///
///     tui.restore()?;
///     Ok(())
/// }
/// ```
pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    /// Initialize the terminal
    ///
    /// This will:
    /// - Enable raw mode (disable line buffering)
    /// - Enter alternate screen (preserve shell history)
    /// - Create a ratatui Terminal instance
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Get mutable reference to terminal for drawing
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }

    /// Restore terminal to original state
    ///
    /// This will:
    /// - Disable raw mode
    /// - Leave alternate screen
    /// - Show cursor
    pub fn restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for Tui {
    /// Automatically restore terminal on drop
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_creation() {
        // This test requires a real terminal, so we just check compilation
        // In real usage, Tui::new() would be called
    }
}
