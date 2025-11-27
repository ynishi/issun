//! ISSUN-CORE: Engine-Agnostic Game Mechanics Library
//!
//! This crate provides a policy-based design for composable game mechanics.
//! It uses Rust's generic type system to enable zero-cost abstractions while
//! maintaining complete separation from any specific game engine.
//!
//! # Architecture
//!
//! ISSUN-CORE follows a **Policy-Based Design** pattern:
//!
//! 1. **Mechanics**: High-level game systems (e.g., contagion, economy)
//! 2. **Policies**: Trait-based "slots" for customizable behavior
//! 3. **Strategies**: Concrete implementations of policies
//! 4. **Composition**: Combine strategies via generics to create custom mechanics
//!
//! # Design Principles
//!
//! - **Zero-Cost Abstraction**: All composition happens at compile time
//! - **Engine-Agnostic**: No dependencies on Bevy, Unity, Godot, etc.
//! - **Static Dispatch**: No `dyn Trait` - everything is monomorphized
//! - **Composability**: Mix and match strategies to create custom behaviors
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::contagion::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Option 1: Use a preset
//! type MyVirus = ZombieVirus; // Explosive spread + hard to resist
//!
//! // Option 2: Use default generics
//! type SimpleVirus = ContagionMechanic; // LinearSpread + ThresholdProgression<10>
//!
//! // Option 3: Fully customize
//! type CustomVirus = ContagionMechanic<ExponentialSpread, ThresholdProgression<50>>;
//!
//! // 2. Create configuration (shared across all entities)
//! let config = ContagionConfig { base_rate: 0.15 };
//!
//! // 3. Create per-entity state
//! let mut state = SimpleSeverity::default();
//!
//! // 4. Prepare input for this frame
//! let input = ContagionInput {
//!     density: 0.7,      // Population density
//!     resistance: 8,     // Entity's resistance
//!     rng: 0.04,         // Random value
//! };
//!
//! // 5. Create an event emitter
//! # struct TestEmitter { events: Vec<ContagionEvent> }
//! # impl EventEmitter<ContagionEvent> for TestEmitter {
//! #     fn emit(&mut self, event: ContagionEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // 6. Execute the mechanic
//! MyVirus::step(&config, &mut state, input, &mut emitter);
//!
//! // 7. Check results
//! if state.severity > 0 {
//!     println!("Entity is infected! Severity: {}", state.severity);
//! }
//! ```
//!
//! # Module Organization
//!
//! - [`mechanics`]: Core trait definitions and mechanic implementations
//!   - [`mechanics::contagion`]: Disease/infection spreading system
//! - [`prelude`]: Convenient re-exports of commonly used items
//!
//! # For Engine Adapters
//!
//! If you're building an engine adapter (e.g., `issun-bevy`), you'll typically:
//!
//! 1. Query ECS data to construct `Input` structs
//! 2. Call `Mechanic::step()` with that input
//! 3. Listen to emitted events and update the game world
//!
//! See the `issun-bevy` crate for a complete example.

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod mechanics;
pub mod prelude;
