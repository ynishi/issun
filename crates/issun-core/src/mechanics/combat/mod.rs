//! Combat mechanic: Damage calculation and combat resolution system.
//!
//! This module provides a policy-based combat system that can model
//! various types of combat, from classic RPGs to modern action games.
//!
//! # Architecture
//!
//! The combat mechanic follows a **Policy-Based Design**:
//! - The core `CombatMechanic<D, F, E>` is generic over three policies
//! - `D: DamageCalculationPolicy` determines how to calculate base damage
//! - `F: DefensePolicy` determines how defense reduces damage
//! - `E: ElementalPolicy` determines how elemental matchups modify damage
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::combat::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define your combat type
//! type ClassicRPG = CombatMechanic<
//!     LinearDamageCalculation,
//!     SubtractiveDefense,
//!     NoElemental,
//! >;
//!
//! // Or use defaults (same as above)
//! type SimpleCombat = CombatMechanic;
//!
//! // Create configuration
//! let config = CombatConfig::default();
//! let mut state = CombatState::new(100);
//!
//! // Prepare input for this combat
//! let input = CombatInput {
//!     attacker_power: 30,
//!     defender_defense: 10,
//!     attacker_element: None,
//!     defender_element: None,
//! };
//!
//! // Event collector
//! struct VecEmitter(Vec<CombatEvent>);
//! impl EventEmitter<CombatEvent> for VecEmitter {
//!     fn emit(&mut self, event: CombatEvent) {
//!         self.0.push(event);
//!     }
//! }
//! let mut emitter = VecEmitter(vec![]);
//!
//! // Execute combat
//! SimpleCombat::step(&config, &mut state, input, &mut emitter);
//!
//! // Check results
//! assert_eq!(state.current_hp, 80); // 100 - (30 - 10) = 80
//! ```
//!
//! # More Examples
//!
//! ## Elemental Combat (Pokémon-style)
//!
//! ```
//! use issun_core::mechanics::combat::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! type ElementalCombat = CombatMechanic<
//!     LinearDamageCalculation,
//!     SubtractiveDefense,
//!     ElementalAffinity,  // Fire > Ice > Water > Fire
//! >;
//!
//! let config = CombatConfig::default();
//! let mut state = CombatState::new(100);
//! let input = CombatInput {
//!     attacker_power: 30,
//!     defender_defense: 10,
//!     attacker_element: Some(Element::Fire),
//!     defender_element: Some(Element::Ice),  // Weakness!
//! };
//!
//! struct VecEmitter(Vec<CombatEvent>);
//! impl EventEmitter<CombatEvent> for VecEmitter {
//!     fn emit(&mut self, event: CombatEvent) { self.0.push(event); }
//! }
//! let mut emitter = VecEmitter(vec![]);
//!
//! ElementalCombat::step(&config, &mut state, input, &mut emitter);
//!
//! // (30 - 10) * 2.0 = 40 damage (super effective!)
//! assert_eq!(state.current_hp, 60);
//! ```
//!
//! ## Modern Action RPG
//!
//! ```
//! use issun_core::mechanics::combat::prelude::*;
//!
//! type ModernARPG = CombatMechanic<
//!     ScalingDamageCalculation,  // Damage scales exponentially
//!     PercentageReduction,        // Armor reduces by %
//!     ElementalAffinity,          // Elemental system
//! >;
//! ```
//!
//! ## Strategy SIM (Fire Emblem-style)
//!
//! ```
//! use issun_core::mechanics::combat::prelude::*;
//!
//! type TacticalSRPG = CombatMechanic<
//!     LinearDamageCalculation,
//!     SubtractiveDefense,
//!     NoElemental,  // Could use WeaponTriangle in the future
//! >;
//! ```
//!
//! # Module Organization
//!
//! - `types`: Core data structures (Config, State, Input, Event, Element)
//! - `policies`: Policy trait definitions
//! - `strategies`: Concrete implementations of policies
//!   - `damage`: Damage calculation strategies
//!   - `defense`: Defense application strategies
//!   - `elemental`: Elemental affinity strategies
//! - `mechanic`: The `CombatMechanic<D, F, E>` implementation
//! - `prelude`: Convenient re-exports

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Convenience re-exports
pub mod prelude;

// Re-export core types
pub use mechanic::CombatMechanic;
pub use policies::{DamageCalculationPolicy, DefensePolicy, ElementalPolicy};
pub use types::{CombatConfig, CombatEvent, CombatInput, CombatState, Element};
