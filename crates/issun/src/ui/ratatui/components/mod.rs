//! Ratatui component implementations
//!
//! This module provides concrete implementations of composable UI components
//! for the ratatui backend.
//!
//! # Component Architecture
//!
//! Components are higher-level UI abstractions that combine data access and rendering.
//! They are trait-based and composable, making it easy to build complex UIs from
//! smaller, reusable pieces.
//!
//! # Available Components
//!
//! - **HeaderComponent**: Turn counter and game mode display
//! - **DistrictsComponent**: Scrollable list of locations with stats
//! - **StatisticsComponent**: Statistics panel with key metrics
//! - **LogComponent**: Event log with control hints
//!
//! # Usage Example
//!
//! ```ignore
//! use issun::ui::ratatui::components::*;
//! use issun::ui::core::{Component, MultiResourceComponent};
//! use issun::ui::ratatui::RatatuiLayer;
//! use issun::ui::layer::UILayoutPresets;
//!
//! // In your render function:
//! fn render_game(frame: &mut Frame, resources: &ResourceContext, selected: usize) {
//!     // Create layout
//!     let layout = RatatuiLayer::three_panel();
//!     let chunks = layout.apply(frame.area());
//!
//!     // Render header
//!     let header = HeaderComponent::<GameContext>::new();
//!     if let Some(widget) = header.render(resources) {
//!         frame.render_widget(widget, chunks[0]);
//!     }
//!
//!     // Render districts
//!     let districts = DistrictsComponent::<CityMap>::new();
//!     if let Some(widget) = districts.render_with_selection(resources, selected) {
//!         frame.render_widget(widget, chunks[1]);
//!     }
//!
//!     // Render log
//!     let log = LogComponent::<GameLog>::new();
//!     if let Some(widget) = log.render_multi(resources) {
//!         frame.render_widget(widget, chunks[2]);
//!     }
//! }
//! ```
//!
//! # Implementing Component Traits
//!
//! To use these components with your game data, implement the required traits:
//!
//! ```ignore
//! use issun::ui::ratatui::components::*;
//!
//! // For HeaderComponent
//! impl HeaderContext for MyGameContext {
//!     fn turn(&self) -> u32 { self.current_turn }
//!     fn max_turns(&self) -> u32 { self.total_turns }
//!     fn mode(&self) -> String { format!("{:?}", self.game_mode) }
//! }
//!
//! // For DistrictsComponent
//! impl DistrictData for MyDistrict {
//!     fn id(&self) -> &str { &self.district_id }
//!     fn name(&self) -> &str { &self.district_name }
//!     fn format_line(&self) -> String {
//!         format!("{}: {} citizens", self.name(), self.population)
//!     }
//! }
//!
//! impl DistrictsProvider for MyCityMap {
//!     type District = MyDistrict;
//!     fn districts(&self) -> &[Self::District] { &self.all_districts }
//! }
//! ```

pub mod districts;
pub mod header;
pub mod log;
pub mod statistics;

pub use districts::{DistrictData, DistrictsComponent, DistrictsProvider};
pub use header::{HeaderComponent, HeaderContext};
pub use log::{LogComponent, LogProvider};
pub use statistics::{StatisticsComponent, StatisticsProvider};
