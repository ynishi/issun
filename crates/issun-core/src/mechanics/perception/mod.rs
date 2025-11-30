//! Perception mechanic: Information accuracy and confidence.
//!
//! This module provides a policy-based system for modeling how entities
//! perceive the world, distinguishing between "ground truth" (absolute reality)
//! and "perceived reality" (what an observer believes).
//!
//! # Key Insight: Information is Uncertain
//!
//! In strategy games, players rarely have perfect information. This mechanic
//! models the gap between reality and perception:
//!
//! - **Accuracy**: How close perceived values are to ground truth
//! - **Confidence**: How reliable the information is (decays over time)
//! - **Delay**: How long until information reaches the observer
//! - **Noise**: Distortion applied based on accuracy
//!
//! # Architecture
//!
//! The perception mechanic follows **Policy-Based Design**:
//! - The core `PerceptionMechanic<P>` is generic over `PerceptionPolicy`
//! - `P: PerceptionPolicy` determines how accuracy and noise are calculated
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::perception::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define perception type (using default FogOfWar policy)
//! type FogOfWar = PerceptionMechanic;
//!
//! // Create configuration
//! let config = PerceptionConfig::default();
//! let mut state = PerceptionState::default();
//!
//! // Prepare input: scout observing enemy troops
//! let input = PerceptionInput {
//!     ground_truth: GroundTruth::quantity(1000),
//!     fact_id: FactId("enemy_troops".into()),
//!     observer: ObserverStats {
//!         entity_id: "scout".into(),
//!         capability: 0.8,
//!         range: 100.0,
//!         tech_bonus: 1.0,
//!         traits: vec![],
//!     },
//!     target: TargetStats {
//!         entity_id: "enemy_army".into(),
//!         concealment: 0.3,
//!         stealth_bonus: 1.0,
//!         environmental_bonus: 1.0,
//!         traits: vec![],
//!     },
//!     distance: 50.0,
//!     rng: 0.5,
//!     current_tick: 100,
//! };
//!
//! // Event collector
//! # struct TestEmitter { events: Vec<PerceptionEvent> }
//! # impl EventEmitter<PerceptionEvent> for TestEmitter {
//! #     fn emit(&mut self, event: PerceptionEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! FogOfWar::step(&config, &mut state, input, &mut emitter);
//!
//! // Scout perceives ~1000 troops (with noise based on accuracy)
//! assert!(state.perception.is_some());
//! assert!(state.accuracy > 0.5);
//! ```
//!
//! # Use Cases
//!
//! - **Fog of War**: Distance-based visibility in strategy games
//! - **Intelligence Networks**: Spy networks providing information with varying accuracy
//! - **Sensor Systems**: Radar/sonar detection with noise and delay
//! - **Market Information**: Price discovery with information asymmetry
//! - **Social Perception**: Reputation and relationship information

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Re-export core types
pub use mechanic::{PerceptionMechanic, SimplePerceptionMechanic};
pub use policies::PerceptionPolicy;
pub use types::{
    DetectionFailureReason, FactId, GroundTruth, ObserverId, ObserverStats, ObserverTrait,
    Perception, PerceptionConfig, PerceptionEvent, PerceptionInput, PerceptionState, TargetId,
    TargetStats, TargetTrait,
};

/// Prelude module for convenient imports.
///
/// Import everything needed to use the perception mechanic:
///
/// ```
/// use issun_core::mechanics::perception::prelude::*;
/// ```
pub mod prelude {
    pub use super::mechanic::{PerceptionMechanic, SimplePerceptionMechanic};
    pub use super::policies::PerceptionPolicy;
    pub use super::strategies::FogOfWarPolicy;
    pub use super::types::{
        DetectionFailureReason, FactId, GroundTruth, ObserverId, ObserverStats, ObserverTrait,
        Perception, PerceptionConfig, PerceptionEvent, PerceptionInput, PerceptionState, TargetId,
        TargetStats, TargetTrait,
    };
}
