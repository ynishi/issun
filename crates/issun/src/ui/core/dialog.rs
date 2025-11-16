//! Dialog widget trait definition
//!
//! Provides modal dialog functionality with title, message, and buttons.

use super::widget::Widget;

/// Dialog widget interface
///
/// Provides modal dialog functionality with title, message, and buttons.
/// Backend implementations should implement this trait.
pub trait Dialog: Widget {
    /// Get selected button index
    fn selected_button(&self) -> usize;

    /// Move to previous button
    fn prev_button(&mut self);

    /// Move to next button
    fn next_button(&mut self);
}
