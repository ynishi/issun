//! Execution characteristics hints for mechanics.
//!
//! This module defines marker types that mechanics use to communicate their
//! preferred execution model to ECS adapters (like issun-bevy).
//!
//! # Design Philosophy
//!
//! - **Hints, not requirements**: These are compile-time suggestions, not hard constraints
//! - **Engine-agnostic**: issun-core declares preferences, adapters interpret them
//! - **Zero-cost**: All types are zero-sized markers resolved at compile time
//!
//! # Usage
//!
//! Mechanics declare their execution characteristics via the `Execution` associated type:
//!
//! ```ignore
//! impl Mechanic for CombatMechanic {
//!     type Execution = ParallelSafe;
//!     // ...
//! }
//! ```
//!
//! ECS adapters (issun-bevy) read these hints to schedule systems appropriately:
//!
//! ```ignore
//! if M::Execution::PARALLEL_SAFE {
//!     app.add_systems(Update, system::<M>);
//! } else {
//!     app.add_systems(Update, system::<M>.in_set(SequentialSet));
//! }
//! ```

/// Trait for execution characteristic hints.
///
/// This trait is implemented by zero-sized marker types that communicate
/// how a mechanic prefers to be executed in an ECS environment.
///
/// # Associated Constants
///
/// - `PARALLEL_SAFE`: Whether this mechanic can safely run in parallel with others
/// - `PREFERRED_SCHEDULE`: Optional hint for which schedule/set to use
/// - `ORDERING_HINT`: Optional hint for relative ordering (e.g., "after:health_regen")
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::execution::{ExecutionHint, ParallelSafe};
///
/// // Check if a mechanic is parallel-safe
/// assert!(ParallelSafe::PARALLEL_SAFE);
/// assert_eq!(ParallelSafe::PREFERRED_SCHEDULE, None);
/// ```
pub trait ExecutionHint {
    /// Whether this mechanic can safely run in parallel with other mechanics.
    ///
    /// - `true`: No data races or ordering dependencies (default)
    /// - `false`: Requires sequential execution or specific ordering
    const PARALLEL_SAFE: bool = true;

    /// Preferred schedule or system set name.
    ///
    /// This is an optional hint for which Bevy schedule or system set
    /// this mechanic should be added to.
    ///
    /// Examples: `"Update"`, `"FixedUpdate"`, `"PostUpdate"`
    const PREFERRED_SCHEDULE: Option<&'static str> = None;

    /// Ordering hint relative to other systems.
    ///
    /// Format: `"after:label"` or `"before:label"`
    ///
    /// Examples:
    /// - `"after:health_regen"` - Run after health regeneration
    /// - `"before:death_check"` - Run before death checking
    const ORDERING_HINT: Option<&'static str> = None;
}

// ============================================================================
// Standard ExecutionHint Implementations
// ============================================================================

/// Parallel-safe execution (default).
///
/// Use this when your mechanic:
/// - Only reads/writes to the entity it's processing
/// - Has no dependencies on other mechanics' execution order
/// - Can safely run concurrently with other mechanics
///
/// # Examples
///
/// ```ignore
/// impl Mechanic for SimpleDamageMechanic {
///     type Execution = ParallelSafe; // Can run in parallel
///     // ...
/// }
/// ```
///
/// Most mechanics should use this as it allows maximum parallelism.
pub struct ParallelSafe;

impl ExecutionHint for ParallelSafe {
    const PARALLEL_SAFE: bool = true;
}

/// Sequential execution with ordering dependency.
///
/// Use this as a base for creating specific ordering constraints.
/// For common patterns, use the provided marker types or define your own.
///
/// # Examples
///
/// ```ignore
/// // Define a custom ordering
/// pub struct AfterHealthRegen;
/// impl ExecutionHint for AfterHealthRegen {
///     const PARALLEL_SAFE: bool = false;
///     const ORDERING_HINT: Option<&'static str> = Some("health_regen");
/// }
///
/// impl Mechanic for CombatMechanic {
///     type Execution = AfterHealthRegen;
///     // ...
/// }
/// ```
pub struct SequentialAfter;

impl ExecutionHint for SequentialAfter {
    const PARALLEL_SAFE: bool = false;
}

/// Sequential execution before other systems.
///
/// Similar to `SequentialAfter`, but indicates this mechanic should
/// run before certain other systems.
pub struct SequentialBefore;

impl ExecutionHint for SequentialBefore {
    const PARALLEL_SAFE: bool = false;
}

/// Transactional execution.
///
/// Use this when your mechanic:
/// - Reads from or writes to multiple entities
/// - Requires consistent view of the world (no partial updates)
/// - Should not interleave with other mechanics
///
/// # Examples
///
/// ```ignore
/// // Propagation reads from neighbors, needs consistency
/// impl Mechanic for PropagationMechanic {
///     type Execution = Transactional;
///     // ...
/// }
/// ```
///
/// This is a stronger constraint than `SequentialAfter` - it suggests
/// the mechanic should run in complete isolation.
pub struct Transactional;

impl ExecutionHint for Transactional {
    const PARALLEL_SAFE: bool = false;
    const PREFERRED_SCHEDULE: Option<&'static str> = Some("Sequential");
}

/// Fixed timestep execution.
///
/// Use this when your mechanic requires a fixed timestep (e.g., physics simulation).
///
/// # Examples
///
/// ```ignore
/// impl Mechanic for PhysicsMechanic {
///     type Execution = FixedTimestep;
///     // ...
/// }
/// ```
pub struct FixedTimestep;

impl ExecutionHint for FixedTimestep {
    const PARALLEL_SAFE: bool = true;
    const PREFERRED_SCHEDULE: Option<&'static str> = Some("FixedUpdate");
}

/// Post-update execution.
///
/// Use this when your mechanic should run after all regular updates
/// (e.g., cleanup, logging, analytics).
///
/// # Examples
///
/// ```ignore
/// impl Mechanic for LoggingMechanic {
///     type Execution = PostUpdate;
///     // ...
/// }
/// ```
pub struct PostUpdate;

impl ExecutionHint for PostUpdate {
    const PARALLEL_SAFE: bool = true;
    const PREFERRED_SCHEDULE: Option<&'static str> = Some("PostUpdate");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_safe_constants() {
        assert!(ParallelSafe::PARALLEL_SAFE);
        assert_eq!(ParallelSafe::PREFERRED_SCHEDULE, None);
        assert_eq!(ParallelSafe::ORDERING_HINT, None);
    }

    #[test]
    fn test_sequential_after() {
        assert!(!SequentialAfter::PARALLEL_SAFE);
        assert_eq!(SequentialAfter::ORDERING_HINT, None);
    }

    #[test]
    fn test_sequential_before() {
        assert!(!SequentialBefore::PARALLEL_SAFE);
    }

    #[test]
    fn test_transactional() {
        assert!(!Transactional::PARALLEL_SAFE);
        assert_eq!(Transactional::PREFERRED_SCHEDULE, Some("Sequential"));
    }

    #[test]
    fn test_fixed_timestep() {
        assert!(FixedTimestep::PARALLEL_SAFE);
        assert_eq!(FixedTimestep::PREFERRED_SCHEDULE, Some("FixedUpdate"));
    }

    #[test]
    fn test_post_update() {
        assert!(PostUpdate::PARALLEL_SAFE);
        assert_eq!(PostUpdate::PREFERRED_SCHEDULE, Some("PostUpdate"));
    }

    #[test]
    fn test_zero_sized() {
        // All execution hints should be zero-sized
        assert_eq!(std::mem::size_of::<ParallelSafe>(), 0);
        assert_eq!(std::mem::size_of::<SequentialAfter>(), 0);
        assert_eq!(std::mem::size_of::<SequentialBefore>(), 0);
        assert_eq!(std::mem::size_of::<Transactional>(), 0);
        assert_eq!(std::mem::size_of::<FixedTimestep>(), 0);
        assert_eq!(std::mem::size_of::<PostUpdate>(), 0);
    }
}
