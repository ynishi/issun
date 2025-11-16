//! Ratatui backend implementations for ISSUN UI widgets
//!
//! This module provides concrete widget implementations using the ratatui TUI library.

pub mod tui;
pub mod menu;
pub mod gauge;
pub mod modal;
// pub mod dialog;  // TODO: Migrate from old structure
// pub mod stats_panel;  // TODO: Add
// pub mod log_viewer;  // TODO: Add

pub use tui::Tui;
pub use menu::MenuWidget;
pub use gauge::{GaugeWidget, ratio_color};
pub use modal::{ModalWidget, centered_rect};
