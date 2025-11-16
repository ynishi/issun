//! UI modules for ISSUN
//!
//! # Structure
//!
//! - `core`: Abstract widget trait definitions (backend-independent)
//! - `ratatui`: Ratatui backend implementations for widgets
//! - `tui`: Terminal initialization and management
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
pub mod tui;
pub mod title_screen;
pub mod ascii_art;

// Re-exports for convenience
pub use core::{Widget, InputEvent};
pub use tui::Tui;
pub use title_screen::{TitleScreenAsset, AsciiFont, TitleScreenService};
