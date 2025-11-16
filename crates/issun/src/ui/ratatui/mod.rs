//! Ratatui backend implementations for ISSUN UI widgets
//!
//! This module provides concrete widget implementations using the ratatui TUI library.

pub mod menu;
// pub mod dialog;  // TODO: Migrate from old structure
// pub mod stats_panel;  // TODO: Add
// pub mod log_viewer;  // TODO: Add

pub use menu::MenuWidget;
