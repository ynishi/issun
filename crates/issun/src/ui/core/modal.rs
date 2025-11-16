//! Modal widget trait definition
//!
//! Provides popup/overlay functionality for displaying modal dialogs.

use super::widget::Widget;

/// Modal widget interface
///
/// Provides popup/overlay functionality for modal dialogs, notifications, etc.
/// Backend implementations should implement this trait.
///
/// Note: Builder methods like `new()`, `with_title()`, `with_size()` etc.
/// should be implemented on the concrete type, not as trait methods.
pub trait Modal: Widget {
    /// Check if modal is currently visible
    fn is_visible(&self) -> bool;

    /// Show the modal
    fn show(&mut self);

    /// Hide the modal
    fn hide(&mut self);

    /// Toggle visibility
    fn toggle(&mut self) {
        if self.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }
}
