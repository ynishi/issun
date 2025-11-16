//! UI modules for ISSUN
//!
//! # Structure
//!
//! - `core`: Abstract widget trait definitions (backend-independent)
//! - `ratatui`: Ratatui backend implementations for widgets (including Tui)
//! - `input`: Input polling utilities for game loops
//! - `title_screen`: Auto-generated title screen system
//! - `ascii_art`: ASCII art presets
//!
//! # Usage
//!
//! ```ignore
//! use issun::ui::Tui;
//! use issun::ui::ratatui::MenuWidget;
//! use issun::ui::core::Widget;
//!
//! let mut tui = Tui::new()?;
//! let menu = MenuWidget::new(vec!["Start".into(), "Quit".into()]);
//! ```

pub mod core;
pub mod ratatui;
pub mod input;
pub mod title;

// Re-exports for convenience
pub use core::{Widget, InputEvent};
pub use ratatui::Tui;
pub use title::title_screen::{TitleScreenAsset, AsciiFont, TitleScreenService};
