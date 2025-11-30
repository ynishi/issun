//! Spatial Plugin - Unified Space and Topology System
//!
//! This plugin integrates issun-core's policy-based spatial mechanic with Bevy's ECS.
//!
//! # Features
//!
//! - Graph-based and grid-based topology support
//! - Distance calculation (Euclidean, Manhattan, Fixed)
//! - Occupancy management
//! - Movement validation
//! - Spatial queries (neighbors, distance, can_move)
//!
//! # Example
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::spatial::*;
//!
//! App::new()
//!     .add_plugins(SpatialPlugin)
//!     .run();
//! ```

pub mod plugin;
pub mod systems;
pub mod types;

pub use plugin::SpatialPlugin;
pub use types::{
    SpatialGraphResource, SpatialLocation, SpatialQueryRequest, SpatialQueryResult,
};
