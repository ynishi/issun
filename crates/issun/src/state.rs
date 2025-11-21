//! State management for game runtime state
//!
//! States store mutable game state that changes during gameplay.
//! Unlike Resources (read-only), States are save/load targets.
//!
//! # Design Principles
//!
//! - **Resources**: Read-only asset/config definitions (PolicyDefinitions, TerritoryDefinitions)
//! - **States**: Mutable runtime state (PolicyState, TerritoryState)
//! - **Save/Load**: Only States are serialized for game saves
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//!
//! // Define state type
//! #[derive(State, Serialize, Deserialize)]
//! pub struct TerritoryState {
//!     control: HashMap<TerritoryId, f32>,
//!     development: HashMap<TerritoryId, u32>,
//! }
//!
//! // Register state during Plugin setup
//! impl Plugin for TerritoryPlugin {
//!     fn setup(&self, ctx: &mut Context) {
//!         ctx.states.register(TerritoryState::new());
//!     }
//! }
//!
//! // Access state in Systems
//! fn system(ctx: &Context) {
//!     let state = ctx.states.get_mut::<TerritoryState>().unwrap();
//!     state.control.insert(territory_id, 0.5);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Marker trait for types that can be stored as game state
///
/// States must be `Send + Sync + Serialize + Deserialize` for:
/// - Thread-safe access
/// - Save/Load functionality
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct PlayerState {
///     hp: i32,
///     level: u32,
/// }
///
/// impl State for PlayerState {}
/// ```
pub trait State: Send + Sync + 'static {
    /// State type name (for debugging)
    fn state_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// State registry for mutable game state
///
/// Stores runtime game state that changes during gameplay.
/// All states are save/load targets.
///
/// # Design
///
/// - **Initialization (GameBuilder/Plugin)**: Can insert states via `register()`
/// - **Runtime (Systems)**: Mutable access via `get_mut()`
/// - **Save/Load**: Serialize/deserialize all states
///
/// # Example
///
/// ```ignore
/// use issun::state::States;
///
/// let mut states = States::new();
///
/// // Register state
/// states.register(PlayerState { hp: 100, level: 1 });
///
/// // Access state
/// if let Some(player) = states.get_mut::<PlayerState>() {
///     player.hp -= 10;
/// }
/// ```
pub struct States {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl States {
    /// Create a new empty state registry
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Register a state into the registry
    ///
    /// This should be called during game initialization (e.g., in GameBuilder or Plugin setup).
    /// If a state of the same type already exists, it will be replaced.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In Plugin setup
    /// ctx.states.register(TerritoryState::new());
    /// ctx.states.register(PolicyState::new());
    /// ```
    pub fn register<T: State>(&mut self, state: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(state));
    }

    /// Get an immutable reference to a state
    ///
    /// Returns `None` if the state doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(state) = ctx.states.get::<TerritoryState>() {
    ///     println!("Control: {:?}", state.control);
    /// }
    /// ```
    pub fn get<T: State>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a state
    ///
    /// Returns `None` if the state doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(state) = ctx.states.get_mut::<TerritoryState>() {
    ///     state.control.insert(territory_id, 0.5);
    /// }
    /// ```
    pub fn get_mut<T: State>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Check if a state exists
    ///
    /// # Example
    ///
    /// ```ignore
    /// if ctx.states.contains::<TerritoryState>() {
    ///     // State is registered
    /// }
    /// ```
    pub fn contains<T: State>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of registered states
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Remove a state from the registry
    ///
    /// Returns the removed state if it existed.
    ///
    /// This is only available within the crate for internal use.
    #[allow(dead_code)]
    pub(crate) fn remove<T: State>(&mut self) -> Option<T> {
        self.data
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Clear all states
    ///
    /// This is only available within the crate for internal use.
    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for States {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestState {
        value: i32,
    }

    impl State for TestState {}

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct AnotherState {
        name: String,
    }

    impl State for AnotherState {}

    #[test]
    fn test_states_new() {
        let states = States::new();
        assert_eq!(states.len(), 0);
        assert!(states.is_empty());
    }

    #[test]
    fn test_register_and_get() {
        let mut states = States::new();
        states.register(TestState { value: 42 });

        let state = states.get::<TestState>();
        assert!(state.is_some());
        assert_eq!(state.unwrap().value, 42);
    }

    #[test]
    fn test_get_nonexistent() {
        let states = States::new();
        let state = states.get::<TestState>();
        assert!(state.is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut states = States::new();
        states.register(TestState { value: 10 });

        if let Some(state) = states.get_mut::<TestState>() {
            state.value = 20;
        }

        assert_eq!(states.get::<TestState>().unwrap().value, 20);
    }

    #[test]
    fn test_multiple_types() {
        let mut states = States::new();
        states.register(TestState { value: 10 });
        states.register(AnotherState {
            name: "Test".to_string(),
        });

        assert!(states.get::<TestState>().is_some());
        assert!(states.get::<AnotherState>().is_some());
        assert_eq!(states.get::<TestState>().unwrap().value, 10);
        assert_eq!(states.get::<AnotherState>().unwrap().name, "Test");
    }

    #[test]
    fn test_replace() {
        let mut states = States::new();
        states.register(TestState { value: 1 });
        states.register(TestState { value: 2 }); // Replace

        let state = states.get::<TestState>();
        assert_eq!(state.unwrap().value, 2);
    }

    #[test]
    fn test_contains() {
        let mut states = States::new();
        states.register(TestState { value: 5 });

        assert!(states.contains::<TestState>());
        assert!(!states.contains::<AnotherState>());
    }

    #[test]
    fn test_remove() {
        let mut states = States::new();
        states.register(TestState { value: 99 });

        let removed = states.remove::<TestState>();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 99);

        // Should be gone now
        assert!(!states.contains::<TestState>());
    }

    #[test]
    fn test_len_and_clear() {
        let mut states = States::new();
        assert_eq!(states.len(), 0);
        assert!(states.is_empty());

        states.register(TestState { value: 1 });
        states.register(AnotherState {
            name: "DB".to_string(),
        });
        assert_eq!(states.len(), 2);
        assert!(!states.is_empty());

        states.clear();
        assert_eq!(states.len(), 0);
        assert!(states.is_empty());
    }

    #[test]
    fn test_state_type_name() {
        let state = TestState { value: 42 };
        let type_name = state.state_type();
        assert!(type_name.contains("TestState"));
    }
}
