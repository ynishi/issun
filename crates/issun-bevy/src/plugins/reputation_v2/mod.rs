//! Reputation Plugin V2 - Policy-Based Design
//!
//! This plugin integrates issun-core's policy-based reputation mechanic with Bevy's ECS.
//! It demonstrates how to adapt pure, engine-agnostic scalar value management to a game engine.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ issun-bevy (Adapter Layer)                                  │
//! │                                                              │
//! │  ┌──────────────┐      ┌───────────────────┐               │
//! │  │   ECS Data   │─────▶│ ReputationInput   │               │
//! │  │ (Component)  │      │  (Pure Data)      │               │
//! │  └──────────────┘      └───────────────────┘               │
//! │                              │                              │
//! │                              ▼                              │
//! │                    ┌──────────────────┐                    │
//! │                    │ Mechanic::step() │  ◀─── issun-core  │
//! │                    │  (Pure Logic)    │                    │
//! │                    └──────────────────┘                    │
//! │                              │                              │
//! │                              ▼                              │
//! │                       ┌─────────────────┐                  │
//! │                       │ ReputationState │                  │
//! │                       │    + Events     │                  │
//! │                       └─────────────────┘                  │
//! │                              │                              │
//! │                              ▼                              │
//! │  ┌──────────────┐      ┌──────────────┐                   │
//! │  │   ECS Data   │◀─────│   Adapter    │                   │
//! │  │  (Updated)   │      │    Layer     │                   │
//! │  └──────────────┘      └──────────────┘                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Key Design Principles
//!
//! 1. **Separation of Concerns**:
//!    - `issun-core`: Pure reputation logic (no ECS, no Bevy)
//!    - `issun-bevy`: Adapter layer (queries, components, messages)
//!
//! 2. **Zero-Cost Abstraction**:
//!    - All policy composition happens at compile time
//!    - No dynamic dispatch (`dyn Trait`)
//!    - Equivalent performance to hand-written code
//!
//! 3. **Compile-Time Configuration**:
//!    - Different reputation systems = different generic type parameters
//!    - Type safety: invalid combinations caught by compiler
//!
//! # Usage
//!
//! ## Basic Setup
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::reputation_v2::ReputationPluginV2;
//! use issun_core::mechanics::reputation::prelude::*;
//!
//! // Define your reputation type
//! type NPCFavorability = ReputationMechanic<
//!     LinearChange,
//!     NoDecay,
//!     HardClamp,
//! >;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ReputationPluginV2::<NPCFavorability>::default())
//!     .run();
//! ```
//!
//! ## Spawning Reputation Entities
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::reputation_v2::ReputationValue;
//!
//! fn spawn_npc(mut commands: Commands) {
//!     commands.spawn((
//!         Name::new("Village Elder"),
//!         ReputationValue::new(50.0), // Neutral starting reputation
//!     ));
//! }
//! ```
//!
//! ## Requesting Reputation Changes
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::reputation_v2::ReputationChangeRequested;
//!
//! fn player_helps_npc(
//!     mut commands: Commands,
//!     npc: Query<Entity, With<NPC>>,
//! ) {
//!     let npc_entity = npc.single();
//!
//!     // Good deed increases reputation
//!     commands.write_message(ReputationChangeRequested {
//!         entity: npc_entity,
//!         delta: 10.0,
//!         elapsed_time: 0,
//!     });
//! }
//! ```
//!
//! # Different Reputation Systems
//!
//! ## Skill Progression (Diminishing Returns)
//!
//! ```ignore
//! type SkillProgression = ReputationMechanic<
//!     LogarithmicChange,  // Harder to level up at high levels
//!     NoDecay,
//!     HardClamp,
//! >;
//!
//! App::new()
//!     .add_plugins(ReputationPluginV2::<SkillProgression>::default())
//!     .run();
//! ```
//!
//! ## Durability System (Linear Decay)
//!
//! ```ignore
//! type DurabilitySystem = ReputationMechanic<
//!     LinearChange,
//!     LinearDecay,    // Tools degrade over time
//!     ZeroClamp,      // Can't go below zero
//! >;
//!
//! App::new()
//!     .add_plugins(ReputationPluginV2::<DurabilitySystem>::default())
//!     .run();
//! ```
//!
//! ## Temporary Buff (Exponential Decay)
//!
//! ```ignore
//! type TemporaryBuff = ReputationMechanic<
//!     LinearChange,
//!     ExponentialDecay,  // Fades quickly over time
//!     ZeroClamp,
//! >;
//!
//! App::new()
//!     .add_plugins(ReputationPluginV2::<TemporaryBuff>::default())
//!     .run();
//! ```
//!
//! ## Rank System (Threshold-Based)
//!
//! ```ignore
//! type RankSystem = ReputationMechanic<
//!     ThresholdChange,  // Different gain rates at different ranks
//!     NoDecay,
//!     HardClamp,
//! >;
//!
//! App::new()
//!     .add_plugins(ReputationPluginV2::<RankSystem>::default())
//!     .run();
//! ```

pub mod plugin;
pub mod systems;
pub mod types;

// Re-exports
pub use plugin::ReputationPluginV2;
pub use types::{ReputationChangeRequested, ReputationEventWrapper, ReputationValue};
