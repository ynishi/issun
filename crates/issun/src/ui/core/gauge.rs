//! Gauge widget trait definition
//!
//! Provides progress bar / resource bar functionality (HP, MP, experience, etc.)

use super::widget::Widget;

/// Gauge widget interface
///
/// Provides progress/resource bar visualization with customizable display.
/// Backend implementations should implement this trait.
///
/// Note: Builder methods like `new()`, `with_label()`, `with_ratio()` etc.
/// should be implemented on the concrete type, not as trait methods.
pub trait Gauge: Widget {
    /// Get current ratio (0.0 to 1.0)
    fn ratio(&self) -> f64;

    /// Set ratio (0.0 to 1.0)
    fn set_ratio(&mut self, ratio: f64);

    /// Get label text if any
    fn label(&self) -> Option<&str>;
}
