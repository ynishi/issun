//! ContagionPlugin - Graph-based propagation system
//!
//! Models the spread of information, diseases, trends, and influence through
//! contact networks using graph-based propagation mechanics.
//!
//! # Core Concepts
//!
//! - **Graph Topology**: Static network of nodes connected by edges
//! - **Contagion**: Information/influence/disease that propagates
//! - **Transmission**: Edge-based spreading with probability
//! - **Mutation**: Content changes during transmission
//! - **Credibility Decay**: Information degrades over time
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::contagion::{ContagionPlugin, GraphTopology};
//!
//! let game = GameBuilder::new()
//!     .with_plugin(
//!         ContagionPlugin::new()
//!             .with_topology(world_graph)
//!     )
//!     .build()
//!     .await?;
//! ```

// Module declarations
pub mod config;
pub mod hook;
pub mod plugin;
pub mod service;
pub mod state;
pub mod system;
pub mod topology;
pub mod types;

// Public re-exports
pub use config::ContagionConfig;
pub use hook::{ContagionHook, DefaultContagionHook};
pub use plugin::ContagionPlugin;
pub use service::ContagionService;
pub use state::{Contagion, ContagionState};
pub use system::{ContagionSystem, PropagationReport, SpreadDetail};
pub use topology::{ContagionNode, GraphTopology, NodeType, PropagationEdge};
pub use types::{
    ContagionContent, ContagionId, DiseaseLevel, EdgeId, NodeId, Timestamp, TrendDirection,
};
