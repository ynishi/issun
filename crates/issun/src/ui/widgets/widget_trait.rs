//! Widget trait for ISSUN TUI components

use ratatui::{buffer::Buffer, layout::Rect};

/// Widget trait for TUI components
///
/// All ISSUN widgets implement this trait for consistent rendering
pub trait Widget {
    /// Render the widget into a buffer at the given area
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Get the minimum size required for this widget
    fn min_size(&self) -> (u16, u16) {
        (1, 1)
    }

    /// Handle input (optional, for interactive widgets)
    fn handle_input(&mut self, _key: crossterm::event::KeyCode) -> bool {
        false // Not handled by default
    }
}
