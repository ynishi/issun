//! Game context for ISSUN
//!
//! GameContext holds persistent data that survives scene transitions.

use std::collections::HashMap;
use std::any::Any;

/// Marker trait for game context
///
/// Game context should contain only data that:
/// - Persists across scene transitions
/// - Should be saved/loaded
/// - Is shared between scenes
pub trait GameContext: Send + Sync {
    // Marker trait - no required methods
}

/// Default context implementation
///
/// Provides a simple key-value store for game data
#[derive(Default)]
pub struct Context {
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value into the context
    pub fn insert<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.data.insert(key.into(), Box::new(value));
    }

    /// Get a reference to a value from the context
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key)?.downcast_ref()
    }

    /// Get a mutable reference to a value from the context
    pub fn get_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.data.get_mut(key)?.downcast_mut()
    }

    /// Remove a value from the context
    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// Check if a key exists in the context
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

impl GameContext for Context {}
