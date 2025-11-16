//! Log viewer widget trait definition
//!
//! Displays scrollable log messages.

use super::widget::Widget;

/// Log viewer interface
///
/// Displays scrollable log messages.
/// Backend implementations should implement this trait.
pub trait LogViewer: Widget {
    /// Get current scroll position
    fn scroll_position(&self) -> usize;

    /// Scroll up
    fn scroll_up(&mut self);

    /// Scroll down
    fn scroll_down(&mut self);
}
