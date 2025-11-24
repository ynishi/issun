//! Ratatui backend implementations for ISSUN UI widgets
//!
//! This module provides concrete widget implementations using the ratatui TUI library.

pub mod components;
pub mod gauge;
pub mod layer;
pub mod menu;
pub mod modal;
pub mod theme;
pub mod tui;
// pub mod dialog;  // TODO: Migrate from old structure
// pub mod stats_panel;  // TODO: Add
// pub mod log_viewer;  // TODO: Add

pub use components::{
    DistrictData, DistrictsComponent, DistrictsProvider, HeaderComponent, HeaderContext,
    LogComponent, LogProvider, StatisticsComponent, StatisticsProvider,
};
pub use gauge::{ratio_color, GaugeWidget};
pub use layer::RatatuiLayer;
pub use menu::MenuWidget;
pub use modal::{centered_rect, ModalWidget};
pub use theme::RatatuiTheme;
pub use tui::Tui;
