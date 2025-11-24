//! UI Layer abstraction for composable layouts
//!
//! This module provides a backend-independent abstraction for UI layouts.
//! Concrete implementations are provided in backend-specific modules (e.g., `ui::ratatui`).
//!
//! # Example
//!
//! ```ignore
//! use issun::ui::layer::{UILayer, LayoutDirection, LayoutConstraint};
//! use issun::ui::ratatui::RatatuiLayer;
//!
//! // Define abstract layout
//! let layer = RatatuiLayer::three_panel();
//! let chunks = layer.apply(area);
//! ```

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

/// Layout constraint (backend-independent)
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutConstraint {
    /// Fixed length in cells
    Length(u16),
    /// Percentage of available space (0-100)
    Percentage(u16),
    /// Minimum size, can grow
    Min(u16),
    /// Maximum size, can shrink
    Max(u16),
    /// Proportional ratio (e.g., 1:2:1)
    Ratio(u32, u32),
}

/// UI Layer trait for composable layouts
///
/// This trait defines a backend-independent interface for creating and applying layouts.
/// Implementations are provided for specific UI backends (e.g., ratatui).
pub trait UILayer {
    /// The area type used by this backend (e.g., ratatui::layout::Rect)
    type Area: Clone;

    /// Get the layout name for debugging
    fn name(&self) -> &str;

    /// Get the layout direction
    fn direction(&self) -> LayoutDirection;

    /// Get the layout constraints
    fn constraints(&self) -> &[LayoutConstraint];

    /// Apply this layout to the given area, returning sub-areas
    fn apply(&self, area: Self::Area) -> Vec<Self::Area>;

    /// Split the area into multiple regions based on the layout
    ///
    /// This is a convenience method that calls `apply()` and returns the result.
    fn split(&self, area: Self::Area) -> Vec<Self::Area> {
        self.apply(area)
    }
}

/// Common layout patterns
///
/// These methods provide pre-defined layout patterns that can be used across backends.
pub trait UILayoutPresets: UILayer {
    /// Create a three-panel vertical layout (header, content, footer)
    ///
    /// # Layout
    /// ```text
    /// ┌─────────────┐
    /// │   Header    │  3 lines
    /// ├─────────────┤
    /// │             │
    /// │   Content   │  Min 10 lines
    /// │             │
    /// ├─────────────┤
    /// │   Footer    │  12 lines
    /// └─────────────┘
    /// ```
    fn three_panel() -> Self;

    /// Create a two-column horizontal layout
    ///
    /// # Layout
    /// ```text
    /// ┌──────┬──────┐
    /// │      │      │
    /// │ Left │ Right│
    /// │      │      │
    /// └──────┴──────┘
    /// ```
    ///
    /// # Arguments
    /// * `left_percent` - Percentage width for left column (0-100)
    fn two_column(left_percent: u16) -> Self;

    /// Create a sidebar layout (narrow left, wide right)
    ///
    /// # Layout
    /// ```text
    /// ┌──┬────────────┐
    /// │S │            │
    /// │I │   Main     │
    /// │D │            │
    /// │E │            │
    /// └──┴────────────┘
    /// ```
    fn sidebar() -> Self
    where
        Self: Sized,
    {
        Self::two_column(20)
    }

    /// Create a detail layout (wide left, narrow right)
    ///
    /// # Layout
    /// ```text
    /// ┌────────────┬──┐
    /// │            │I │
    /// │   Main     │N │
    /// │            │F │
    /// │            │O │
    /// └────────────┴──┘
    /// ```
    fn detail() -> Self
    where
        Self: Sized,
    {
        Self::two_column(80)
    }
}
