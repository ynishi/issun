//! UI modules for ISSUN
//!
//! # Structure
//!
//! - `core`: Abstract widget trait definitions (backend-independent)
//! - `ratatui`: Ratatui backend implementations for widgets (including Tui)
//! - `input`: Input polling utilities for game loops
//! - `title_screen`: Auto-generated title screen system
//! - `layer`: UI layout abstraction for composable layouts
//! - `theme`: Theme system for consistent styling
//! - `resource_guard`: Safe resource access wrapper
//! - `macros`: Rendering macros for simplified component composition
//!
//! # Usage
//!
//! ```ignore
//! use issun::ui::Tui;
//! use issun::ui::ratatui::*;
//! use issun::ui::core::{Component, Widget};
//! use issun::ui::layer::UILayoutPresets;
//! use issun::{drive, drive_to};
//!
//! let mut tui = Tui::new()?;
//!
//! // Use components with drive! macro for easy rendering
//! fn render_game(frame: &mut Frame, resources: &ResourceContext, selected: usize) {
//!     let header = HeaderComponent::<GameContext>::new();
//!     let districts = DistrictsComponent::<CityMap>::new();
//!     let log = LogComponent::<GameLog>::new();
//!
//!     drive! {
//!         frame: frame,
//!         layout: RatatuiLayer::three_panel().apply(frame.area()),
//!         [
//!             header.render(resources),
//!             districts.render_with_selection(resources, selected),
//!             log.render_multi(resources),
//!         ]
//!     }
//! }
//! ```

pub mod core;
pub mod input;
pub mod layer;
pub mod macros;
pub mod ratatui;
pub mod resource_guard;
pub mod theme;
pub mod title;

// Re-exports for convenience
pub use core::{Component, InputEvent, MultiResourceComponent, Widget};
pub use layer::{LayoutConstraint, LayoutDirection, UILayer, UILayoutPresets};
pub use ratatui::Tui;
pub use resource_guard::{ResourceError, ResourceGuard};
pub use theme::{Emphasis, Theme, ThemeColor, ThemeConfig, ThemePresets};
pub use title::title_screen::{AsciiFont, TitleScreenAsset, TitleScreenService};
