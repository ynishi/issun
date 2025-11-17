//! Resource Registry for global read-only data
//!
//! Resources are shared, read-only data accessible from Systems and Scenes.
//! Unlike Context (save data), Resources contain:
//! - Asset collections (Vec<EnemyAsset>, Vec<ItemAsset>)
//! - Configuration (GameConfig, DisplaySettings)
//! - Lookup tables (SpawnTable, DropTable)
//!
//! # Relationship with Assets
//!
//! - **Asset**: Individual game content (one enemy, one item)
//! - **Resource**: Collections or databases of assets (EnemyDatabase, ItemCatalog)
//!
//! ```text
//! #[derive(Asset)]           ← Individual content
//! struct EnemyAsset { ... }
//!
//! #[derive(Resource)]        ← Collection of assets
//! struct EnemyDatabase {
//!     enemies: Vec<EnemyAsset>
//! }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//!
//! // Define your resource types
//! #[derive(Resource)]
//! pub struct EnemyDatabase {
//!     pub enemies: Vec<EnemyAsset>,
//! }
//!
//! #[derive(Resource, Default)]
//! pub struct GameConfig {
//!     pub fps: u32,
//!     pub difficulty: f32,
//! }
//!
//! // Register resources during initialization
//! let mut resources = Resources::new();
//! resources.register(EnemyDatabase { enemies: vec![...] });
//! resources.register(GameConfig { fps: 60, difficulty: 1.0 });
//!
//! // Access resources in Systems/Scenes (read-only)
//! let enemies = resources.get::<EnemyDatabase>().unwrap();
//! let config = resources.get::<GameConfig>().unwrap();
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Marker trait for types that can be stored as global resources
///
/// Resources must be `Send + Sync + 'static` for thread-safe access.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
///
/// #[derive(Resource)]
/// struct GameConfig {
///     fps: u32,
///     difficulty: f32,
/// }
/// ```
pub trait Resource: Send + Sync + 'static {
    /// Resource type name (for debugging)
    fn resource_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Resource registry for global read-only data
///
/// Uses type-based lookup to store and retrieve resources.
/// Resources should be immutable data that doesn't change during gameplay.
///
/// # Initialization vs Runtime
///
/// - **Initialization (GameBuilder)**: Can insert resources via `add_resource()`
/// - **Runtime (Systems/Scenes)**: Read-only access via `get()`
///
/// This follows the "App State Pattern" (like Axum's State), ensuring
/// resources remain immutable after game initialization.
pub struct Resources {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    /// Create a new empty resource registry
    ///
    /// This is only available within the crate for GameBuilder.
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Register a resource into the registry
    ///
    /// This should only be called during game initialization (e.g., in GameBuilder or Plugin setup).
    /// If a resource of the same type already exists, it will be replaced.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In GameBuilder or Plugin
    /// resources.register(GameConfig { fps: 60, difficulty: 1.0 });
    /// resources.register(EnemyDatabase { enemies: vec![...] });
    /// ```
    ///
    /// # Design Note
    ///
    /// This method is intentionally named `register` (not `insert`) to convey
    /// that it should only be used during initialization, following the
    /// "App State Pattern" where resources are registered once at startup
    /// and remain read-only during runtime.
    pub fn register<T: Resource>(&mut self, resource: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Get an immutable reference to a resource
    ///
    /// Returns `None` if the resource doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In a System or Scene
    /// if let Some(config) = resources.get::<GameConfig>() {
    ///     println!("FPS: {}", config.fps);
    /// }
    /// ```
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Check if a resource exists
    ///
    /// # Example
    ///
    /// ```ignore
    /// if resources.contains::<GameConfig>() {
    ///     // Config is registered
    /// }
    /// ```
    pub fn contains<T: Resource>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of registered resources
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Remove a resource from the registry
    ///
    /// Returns the removed resource if it existed.
    ///
    /// This is only available within the crate for internal use.
    pub(crate) fn remove<T: Resource>(&mut self) -> Option<T> {
        self.data
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Clear all resources
    ///
    /// This is only available within the crate for internal use.
    pub(crate) fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use issun_macros::Resource;

    use super::*;

    // Test manual implementation
    impl Resource for TestConfig {}
    impl Resource for TestDatabase {}

    #[derive(Debug, Clone, PartialEq)]
    struct TestConfig {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestDatabase {
        name: String,
    }

    // Test derive macro implementation
    // Use the actual #[derive(Resource)] macro
    #[derive(Resource, Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    struct DerivedConfig {
        fps: u32,
        difficulty: f32,
    }

    #[derive(crate::Resource, Debug, Clone)]
    #[allow(dead_code)]
    struct DerivedDatabase {
        items: Vec<String>,
    }

    #[test]
    fn test_register_and_get() {
        let mut resources = Resources::new();
        resources.register(TestConfig { value: 42 });

        let config = resources.get::<TestConfig>();
        assert!(config.is_some());
        assert_eq!(config.unwrap().value, 42);
    }

    #[test]
    fn test_get_nonexistent() {
        let resources = Resources::new();
        let config = resources.get::<TestConfig>();
        assert!(config.is_none());
    }

    #[test]
    fn test_multiple_types() {
        let mut resources = Resources::new();
        resources.register(TestConfig { value: 10 });
        resources.register(TestDatabase {
            name: "Test".to_string(),
        });

        assert!(resources.get::<TestConfig>().is_some());
        assert!(resources.get::<TestDatabase>().is_some());
        assert_eq!(resources.get::<TestConfig>().unwrap().value, 10);
        assert_eq!(resources.get::<TestDatabase>().unwrap().name, "Test");
    }

    #[test]
    fn test_replace() {
        let mut resources = Resources::new();
        resources.register(TestConfig { value: 1 });
        resources.register(TestConfig { value: 2 }); // Replace

        let config = resources.get::<TestConfig>();
        assert_eq!(config.unwrap().value, 2);
    }

    #[test]
    fn test_contains() {
        let mut resources = Resources::new();
        resources.register(TestConfig { value: 5 });

        assert!(resources.contains::<TestConfig>());
        assert!(!resources.contains::<TestDatabase>());
    }

    #[test]
    fn test_remove() {
        let mut resources = Resources::new();
        resources.register(TestConfig { value: 99 });

        let removed = resources.remove::<TestConfig>();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 99);

        // Should be gone now
        assert!(!resources.contains::<TestConfig>());
    }

    #[test]
    fn test_len_and_clear() {
        let mut resources = Resources::new();
        assert_eq!(resources.len(), 0);
        assert!(resources.is_empty());

        resources.register(TestConfig { value: 1 });
        resources.register(TestDatabase {
            name: "DB".to_string(),
        });
        assert_eq!(resources.len(), 2);
        assert!(!resources.is_empty());

        resources.clear();
        assert_eq!(resources.len(), 0);
        assert!(resources.is_empty());
    }

    #[test]
    fn test_derived_resource() {
        let mut resources = Resources::new();
        resources.register(DerivedConfig {
            fps: 60,
            difficulty: 1.5,
        });

        let config = resources.get::<DerivedConfig>();
        assert!(config.is_some());
        assert_eq!(config.unwrap().fps, 60);
        assert_eq!(config.unwrap().difficulty, 1.5);
    }

    #[test]
    fn test_resource_type_name() {
        let config = TestConfig { value: 42 };
        let type_name = config.resource_type();
        assert!(type_name.contains("TestConfig"));
    }

    #[test]
    fn test_derive_macro_single_type() {
        let mut resources = Resources::new();
        resources.register(DerivedConfig {
            fps: 60,
            difficulty: 1.5,
        });

        let config = resources.get::<DerivedConfig>();
        assert!(config.is_some());
        assert_eq!(config.unwrap().fps, 60);
    }

    #[test]
    fn test_derive_macro_multiple_types() {
        let mut resources = Resources::new();

        resources.register(DerivedConfig {
            fps: 30,
            difficulty: 2.0,
        });

        resources.register(DerivedDatabase {
            items: vec!["item1".to_string(), "item2".to_string()],
        });

        assert!(resources.contains::<DerivedConfig>());
        assert!(resources.contains::<DerivedDatabase>());

        let db = resources.get::<DerivedDatabase>();
        assert_eq!(db.unwrap().items.len(), 2);
    }
}
