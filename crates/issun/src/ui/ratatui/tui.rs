//! Terminal User Interface wrapper for ISSUN
//!
//! Provides a simple abstraction for terminal initialization and cleanup.
//!
//! # Game Loop Patterns
//!
//! ## Simple Loop (80% use case)
//!
//! Use [`Tui::run_simple_loop`] for typical game loops:
//!
//! ```ignore
//! use issun::ui::{Tui, InputEvent};
//! use std::time::Duration;
//!
//! struct GameState {
//!     counter: u32,
//!     should_quit: bool,
//! }
//!
//! let mut tui = Tui::new()?;
//! let mut state = GameState { counter: 0, should_quit: false };
//!
//! tui.run_simple_loop(
//!     &mut state,
//!     Duration::from_millis(33), // 30 FPS
//!     |frame, state| {
//!         // Render your UI here
//!     },
//!     |state, input| {
//!         match input {
//!             InputEvent::Quit => state.should_quit = true,
//!             _ => {}
//!         }
//!         state.should_quit // Return true to exit loop
//!     },
//!     Some(|state| {
//!         // Optional: Called every tick
//!         state.counter += 1;
//!     }),
//! )?;
//! ```
//!
//! ## Custom Loop (full control)
//!
//! For complex scenarios, implement your own loop:
//!
//! ```
//! # use std::time::{Duration, Instant};
//! # use issun::ui::InputEvent;
//! # struct GameState { should_quit: bool }
//! # fn render_scene(_frame: &mut ratatui::Frame, _state: &GameState) {}
//! # fn handle_input(_state: &mut GameState, _input: InputEvent) {}
//! # fn update_game(_state: &mut GameState) {}
//! #
//! fn custom_game_loop(
//!     tui: &mut issun::ui::Tui,
//!     state: &mut GameState,
//! ) -> std::io::Result<()> {
//!     let tick_rate = Duration::from_millis(33); // 30 FPS
//!     let mut last_tick = Instant::now();
//!
//!     loop {
//!         // Draw
//!         tui.terminal().draw(|frame| {
//!             render_scene(frame, state);
//!         })?;
//!
//!         // Calculate timeout for next tick
//!         let timeout = tick_rate
//!             .checked_sub(last_tick.elapsed())
//!             .unwrap_or_else(|| Duration::from_secs(0));
//!
//!         // Poll input with timeout
//!         let input = issun::ui::input::poll_input(timeout)?;
//!
//!         // Handle input
//!         if input != InputEvent::Other {
//!             handle_input(state, input);
//!         }
//!
//!         // Update game state every tick
//!         if last_tick.elapsed() >= tick_rate {
//!             update_game(state);
//!             last_tick = Instant::now();
//!         }
//!
//!         // Exit condition
//!         if state.should_quit {
//!             break;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

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

    /// Run a simple game loop (80% use case)
    ///
    /// This is a reference implementation of a typical TUI game loop.
    /// For more complex scenarios, implement your own loop (see module docs).
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable game state
    /// * `tick_rate` - Frame rate (e.g., `Duration::from_millis(33)` for 30 FPS)
    /// * `render` - Render callback called every frame
    /// * `on_input` - Input handler, returns `true` to quit
    /// * `on_tick` - Optional tick callback called at `tick_rate` intervals
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::ui::{Tui, InputEvent};
    /// use std::time::Duration;
    ///
    /// struct GameState { should_quit: bool }
    ///
    /// let mut tui = Tui::new()?;
    /// let mut state = GameState { should_quit: false };
    ///
    /// tui.run_simple_loop(
    ///     &mut state,
    ///     Duration::from_millis(33),
    ///     |_frame, _state| { /* render */ },
    ///     |state, input| {
    ///         if input == InputEvent::Quit {
    ///             state.should_quit = true;
    ///         }
    ///         state.should_quit
    ///     },
    ///     None::<fn(&mut GameState)>,
    /// )?;
    /// ```
    pub fn run_simple_loop<S>(
        &mut self,
        state: &mut S,
        tick_rate: Duration,
        mut render: impl FnMut(&mut ratatui::Frame, &S),
        mut on_input: impl FnMut(&mut S, crate::ui::InputEvent) -> bool,
        mut on_tick: Option<impl FnMut(&mut S)>,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            // Draw
            self.terminal.draw(|frame| {
                render(frame, state);
            })?;

            // Calculate timeout for next tick
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // Poll input with timeout
            let input = crate::ui::input::poll_input(timeout)?;

            // Handle input
            if input != crate::ui::InputEvent::Other {
                if on_input(state, input) {
                    break; // Quit requested
                }
            }

            // Tick
            if last_tick.elapsed() >= tick_rate {
                if let Some(ref mut tick_fn) = on_tick {
                    tick_fn(state);
                }
                last_tick = Instant::now();
            }
        }

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
