//! Menu widget trait definition
//!
//! Provides navigation and selection functionality for menu-style UIs.

use super::widget::Widget;

/// Menu widget interface
///
/// Provides navigation and selection functionality for menu-style UIs.
/// Backend implementations should implement this trait.
pub trait Menu: Widget {
    /// Get currently selected index
    fn selected(&self) -> usize;

    /// Get selected item text
    fn selected_item(&self) -> Option<&str>;

    /// Move cursor up
    fn move_up(&mut self);

    /// Move cursor down
    fn move_down(&mut self);
}
