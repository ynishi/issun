//! Stats panel widget trait definition
//!
//! Displays character/entity statistics (HP, Attack, Defense, etc.)

use super::widget::Widget;

/// Stats panel interface
///
/// Displays character/entity statistics (HP, Attack, Defense, etc.)
/// Backend implementations should implement this trait.
///
/// Note: `new()`, `add_stat_with_max()`, and `add_stat()` are builder methods
/// and should be implemented as associated functions/methods on the concrete type,
/// not as trait methods.
pub trait StatsPanel: Widget {
    // This trait is currently empty, as the original trait only had builder methods.
    // If you need to add methods for updating stats dynamically or getting stats,
    // add them here. For example:
    //
    // fn update_stat(&mut self, name: &str, value: i32);
    // fn get_stat(&self, name: &str) -> Option<i32>;
}
