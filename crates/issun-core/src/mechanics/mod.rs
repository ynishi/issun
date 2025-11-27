//! Core trait definitions for the Mechanic system.
//!
//! This module defines the fundamental abstractions that all mechanics must implement.
//! The design is based on Policy-Based Design patterns, allowing for compile-time
//! composition of behaviors with zero runtime overhead.

pub mod combat;
pub mod diplomacy;
pub mod contagion;
pub mod evolution;
pub mod execution;
pub mod propagation;
pub mod state_machine;

// Re-export execution hints for convenience
pub use execution::{ExecutionHint, ParallelSafe, SequentialAfter, Transactional};

/// A trait for emitting events from a mechanic.
///
/// This abstraction allows mechanics to remain engine-agnostic while still
/// communicating state changes to the outside world.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::EventEmitter;
///
/// struct MyEventEmitter {
///     events: Vec<String>,
/// }
///
/// impl EventEmitter<String> for MyEventEmitter {
///     fn emit(&mut self, event: String) {
///         self.events.push(event);
///     }
/// }
/// ```
pub trait EventEmitter<E> {
    /// Emit an event of type E.
    fn emit(&mut self, event: E);
}

/// The core trait that all mechanics must implement.
///
/// A mechanic represents a self-contained piece of game logic that operates on:
/// - `Config`: Static configuration (e.g., base infection rate)
/// - `State`: Mutable state per entity (e.g., infection severity)
/// - `Input`: Per-frame input data (e.g., density, resistance)
/// - `Event`: Events emitted when state changes occur
/// - `Execution`: Execution characteristics hint (e.g., parallel safety, ordering)
///
/// # Type Parameters
///
/// All associated types are determined by the specific mechanic implementation.
/// This allows the mechanic to be completely decoupled from any engine.
///
/// # Design Philosophy
///
/// This trait is designed to support **Policy-Based Design**:
/// - Mechanics are generic over policy traits (e.g., `ContagionMechanic<S, P>`)
/// - All logic is resolved at compile time (static dispatch)
/// - Zero runtime overhead compared to hand-written code
///
/// # Examples
///
/// ```ignore
/// use std::marker::PhantomData;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Define a policy trait
/// trait SpreadPolicy {
///     fn calculate_rate(base_rate: f32, density: f32) -> f32;
/// }
///
/// // A mechanic that is generic over the policy
/// struct MyMechanic<S: SpreadPolicy> {
///     _marker: PhantomData<S>,
/// }
///
/// impl<S: SpreadPolicy> Mechanic for MyMechanic<S> {
///     type Config = MyConfig;
///     type State = MyState;
///     type Input = MyInput;
///     type Event = MyEvent;
///
///     fn step(
///         config: &Self::Config,
///         state: &mut Self::State,
///         input: Self::Input,
///         emitter: &mut impl EventEmitter<Self::Event>,
///     ) {
///         // Pure logic that delegates to policy S
///         let rate = S::calculate_rate(config.base_rate, input.density);
///         // ...
///     }
/// }
/// ```
pub trait Mechanic {
    /// Static configuration for this mechanic (e.g., base rates, thresholds).
    ///
    /// This type should be cheap to clone and typically stored as a resource
    /// in the game engine.
    type Config;

    /// Per-entity mutable state (e.g., health, infection level).
    ///
    /// This type is typically stored as a component on each entity.
    type State;

    /// Per-frame input data (e.g., density, environmental factors).
    ///
    /// This type is constructed fresh each frame from the game world state.
    type Input;

    /// Events that this mechanic can emit.
    ///
    /// Events are used to communicate state changes to the game world
    /// without coupling the mechanic to a specific engine.
    type Event;

    /// Execution characteristics hint for ECS integration.
    ///
    /// This type provides compile-time hints about how this mechanic should be
    /// scheduled in an ECS (e.g., Bevy). It's a **hint**, not a requirement:
    /// - issun-core mechanics declare their preferred execution model
    /// - issun-bevy (or other adapters) interpret these hints for system scheduling
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Combat is parallel-safe
    /// type Execution = ParallelSafe;
    ///
    /// // Propagation reads from multiple entities, prefer sequential
    /// type Execution = Transactional;
    /// ```
    type Execution: ExecutionHint;

    /// Execute one step of the mechanic's logic.
    ///
    /// This is a pure function that:
    /// 1. Reads from `config` (immutable, shared across all entities)
    /// 2. Reads and modifies `state` (mutable, per-entity)
    /// 3. Consumes `input` (per-frame data)
    /// 4. Emits events via `emitter` when state changes occur
    ///
    /// # Design Notes
    ///
    /// - This function should be deterministic given the same inputs
    /// - All randomness should come from `input` (e.g., pre-generated RNG values)
    /// - No I/O or side effects beyond emitting events
    /// - Should be safe to call in parallel for different entities
    ///
    /// # Parameters
    ///
    /// - `config`: Shared configuration for all entities
    /// - `state`: Mutable state for a single entity
    /// - `input`: Frame-specific input data for this entity
    /// - `emitter`: Event emitter for communicating state changes
    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    );
}
