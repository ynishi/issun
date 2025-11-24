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
//!
//! # Usage
//!
//! ```ignore
//! use issun::ui::Tui;
//! use issun::ui::ratatui::MenuWidget;
//! use issun::ui::core::Widget;
//! use issun::ui::layer::{UILayer, UILayoutPresets};
//! use issun::ui::ResourceGuard;
//!
//! let mut tui = Tui::new()?;
//! let menu = MenuWidget::new(vec!["Start".into(), "Quit".into()]);
//!
//! // Use UILayer for layouts
//! let layout = RatatuiLayer::three_panel();
//! let chunks = layout.apply(area);
//!
//! // Use ResourceGuard for safe resource access
//! let ctx = ResourceGuard::new::<GameContext>(resources);
//! ```

pub mod core;
pub mod input;
pub mod layer;
pub mod ratatui;
pub mod resource_guard;
pub mod theme;
pub mod title;

// Re-exports for convenience
pub use core::{InputEvent, Widget};
pub use layer::{LayoutConstraint, LayoutDirection, UILayer, UILayoutPresets};
pub use ratatui::Tui;
pub use resource_guard::{ResourceError, ResourceGuard};
pub use theme::{Emphasis, Theme, ThemeColor, ThemeConfig, ThemePresets};
pub use title::title_screen::{AsciiFont, TitleScreenAsset, TitleScreenService};
