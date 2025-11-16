//! Core UI trait definitions for ISSUN
//!
//! This module contains abstract trait definitions independent of specific TUI libraries.
//! Concrete implementations are provided in backend-specific modules (e.g., `ui::ratatui`).
//!
//! # Structure
//!
//! - `widget`: Base Widget trait and InputEvent
//! - `menu`: Menu widget trait
//! - `dialog`: Dialog widget trait
//! - `stats`: Stats panel widget trait
//! - `log`: Log viewer widget trait
//! - `gauge`: Gauge/progress bar widget trait
//! - `modal`: Modal/popup widget trait

pub mod widget;
pub mod menu;
pub mod dialog;
pub mod stats;
pub mod log;
pub mod gauge;
pub mod modal;

// Re-exports for convenience
pub use widget::{Widget, InputEvent};
pub use menu::Menu;
pub use dialog::Dialog;
pub use stats::StatsPanel;
pub use log::LogViewer;
pub use gauge::Gauge;
pub use modal::Modal;
