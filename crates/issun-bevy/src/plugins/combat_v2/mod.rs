//! Combat Plugin V2 - Policy-Based Design
//!
//! This plugin integrates issun-core's policy-based combat mechanic with Bevy's ECS.
//! It demonstrates how to adapt pure, engine-agnostic logic to a specific game engine.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ issun-bevy (Adapter Layer)                                  │
//! │                                                              │
//! │  ┌──────────────┐      ┌──────────────┐                    │
//! │  │   ECS Data   │─────▶│  CombatInput │                    │
//! │  │ (Components) │      │  (Pure Data) │                    │
//! │  └──────────────┘      └──────────────┘                    │
//! │                              │                              │
//! │                              ▼                              │
//! │                    ┌──────────────────┐                    │
//! │                    │ Mechanic::step() │  ◀─── issun-core  │
//! │                    │  (Pure Logic)    │                    │
//! │                    └──────────────────┘                    │
//! │                              │                              │
//! │                              ▼                              │
//! │                       ┌──────────────┐                     │
//! │                       │ CombatState  │                     │
//! │                       │  + Events    │                     │
//! │                       └──────────────┘                     │
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
//!    - `issun-core`: Pure combat logic (no ECS, no Bevy)
//!    - `issun-bevy`: Adapter layer (queries, components, messages)
//!
//! 2. **Zero-Cost Abstraction**:
//!    - All policy composition happens at compile time
//!    - No dynamic dispatch (`dyn Trait`)
//!    - Equivalent performance to hand-written code
//!
//! 3. **Compile-Time Configuration**:
//!    - Different combat systems = different generic type parameters
//!    - Type safety: invalid combinations caught by compiler
//!
//! # Usage
//!
//! ## Basic Setup
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::combat_v2::CombatPluginV2;
//! use issun_core::mechanics::combat::prelude::*;
//!
//! // Define your combat type
//! type MyGameCombat = CombatMechanic<
//!     LinearDamageCalculation,
//!     SubtractiveDefense,
//!     NoElemental,
//! >;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(CombatPluginV2::<MyGameCombat>::default())
//!     .run();
//! ```
//!
//! ## Spawning Combat Entities
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::combat_v2::{Health, Attack, Defense};
//!
//! fn spawn_knight(mut commands: Commands) {
//!     commands.spawn((
//!         Name::new("Knight"),
//!         Health::new(100),
//!         Attack { power: 30 },
//!         Defense { value: 15 },
//!     ));
//! }
//! ```
//!
//! ## Requesting Damage
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::combat_v2::DamageRequested;
//!
//! fn player_attack(
//!     mut commands: Commands,
//!     player: Query<Entity, With<Player>>,
//!     target: Query<Entity, With<Enemy>>,
//! ) {
//!     let player_entity = player.single();
//!     let enemy_entity = target.single();
//!
//!     commands.write_message(DamageRequested {
//!         attacker: player_entity,
//!         target: enemy_entity,
//!     });
//! }
//! ```
//!
//! # Different Combat Systems
//!
//! ## Elemental Combat (Pokémon-style)
//!
//! ```ignore
//! type ElementalCombat = CombatMechanic<
//!     LinearDamageCalculation,
//!     SubtractiveDefense,
//!     ElementalAffinity,  // Fire > Ice > Water > Fire
//! >;
//!
//! App::new()
//!     .add_plugins(CombatPluginV2::<ElementalCombat>::default())
//!     .run();
//! ```
//!
//! ## Modern Action RPG
//!
//! ```ignore
//! type ModernARPG = CombatMechanic<
//!     ScalingDamageCalculation,  // Damage = attack^1.2
//!     PercentageReduction,        // Defense = % reduction
//!     ElementalAffinity,
//! >;
//!
//! App::new()
//!     .add_plugins(CombatPluginV2::<ModernARPG>::default())
//!     .run();
//! ```

pub mod plugin;
pub mod systems;
pub mod types;

// Re-exports
pub use plugin::CombatPluginV2;
pub use types::{Attack, DamageApplied, DamageRequested, Defense, ElementType, Health};
