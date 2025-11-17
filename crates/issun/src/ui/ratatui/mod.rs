//! Ratatui backend implementations for ISSUN UI widgets
//!
//! This module provides concrete widget implementations using the ratatui TUI library.

pub mod gauge;
pub mod menu;
pub mod modal;
pub mod tui;
// pub mod dialog;  // TODO: Migrate from old structure
// pub mod stats_panel;  // TODO: Add
// pub mod log_viewer;  // TODO: Add

pub use gauge::{ratio_color, GaugeWidget};
pub use menu::MenuWidget;
pub use modal::{centered_rect, ModalWidget};
pub use tui::Tui;
