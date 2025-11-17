//! In-memory store for entities and game objects
//!
//! Provides HashMap-based storage for game entities, assets, and other objects.
//! Useful for managing collections of entities that need to be accessed by ID.

use std::collections::HashMap;
use std::hash::Hash;

/// Generic in-memory store for game objects
///
/// A simple HashMap wrapper that provides common operations for storing and
/// retrieving game objects by ID.
///
/// # Type Parameters
///
/// * `K` - Key type (typically String or &str)
/// * `V` - Value type (Entity, Asset, etc.)
///
/// # Example
///
/// ```
/// use issun::store::Store;
///
/// #[derive(Debug, Clone)]
/// struct Enemy {
///     name: String,
///     hp: i32,
/// }
///
/// let mut enemies = Store::new();
/// enemies.insert("goblin", Enemy { name: "Goblin".into(), hp: 30 });
/// enemies.insert("orc", Enemy { name: "Orc".into(), hp: 50 });
///
/// assert_eq!(enemies.get("goblin").unwrap().hp, 30);
/// assert_eq!(enemies.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct Store<K, V>
where
    K: Eq + Hash,
{
    data: HashMap<K, V>,
}

impl<K, V> Store<K, V>
where
    K: Eq + Hash,
{
    /// Create a new empty store
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create a store with a specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    /// Insert a value into the store
    ///
    /// Returns the previous value if the key already existed.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.data.insert(key, value)
    }

    /// Get a reference to a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    /// Get a mutable reference to a value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(key)
    }

    /// Remove a value from the store
    ///
    /// Returns the removed value if it existed.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }

    /// Check if a key exists in the store
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    /// Get the number of items in the store
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all items from the store
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get an iterator over the store's key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }

    /// Get a mutable iterator over the store's key-value pairs
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.data.iter_mut()
    }

    /// Get an iterator over the store's keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.keys()
    }

    /// Get an iterator over the store's values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.values()
    }

    /// Get a mutable iterator over the store's values
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.values_mut()
    }

    /// Retain only the elements that satisfy the predicate
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.data.retain(f);
    }
}

impl<K, V> Default for Store<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> FromIterator<(K, V)> for Store<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            data: HashMap::from_iter(iter),
        }
    }
}

/// Specialized store for entities with string IDs
///
/// This is a convenience type for the common case of storing entities
/// with String keys.
///
/// # Example
///
/// ```
/// use issun::store::EntityStore;
///
/// #[derive(Debug, Clone)]
/// struct Player {
///     name: String,
///     hp: i32,
/// }
///
/// let mut players = EntityStore::new();
/// players.insert("alice".to_string(), Player { name: "Alice".into(), hp: 100 });
/// players.insert("bob".to_string(), Player { name: "Bob".into(), hp: 90 });
///
/// // Get all alive players
/// let alive: Vec<_> = players.values()
///     .filter(|p| p.hp > 0)
///     .collect();
/// assert_eq!(alive.len(), 2);
/// ```
pub type EntityStore<V> = Store<String, V>;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        name: String,
        hp: i32,
    }

    #[test]
    fn test_store_basic_operations() {
        let mut store = Store::new();

        // Insert
        let entity = TestEntity {
            name: "Goblin".into(),
            hp: 30,
        };
        assert_eq!(store.insert("goblin", entity.clone()), None);
        assert_eq!(store.len(), 1);

        // Get
        assert_eq!(store.get(&"goblin").unwrap().hp, 30);

        // Contains
        assert!(store.contains_key(&"goblin"));
        assert!(!store.contains_key(&"orc"));

        // Update
        store.get_mut(&"goblin").unwrap().hp = 20;
        assert_eq!(store.get(&"goblin").unwrap().hp, 20);

        // Remove
        let removed = store.remove(&"goblin").unwrap();
        assert_eq!(removed.hp, 20);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_store_iteration() {
        let mut store = Store::new();
        store.insert(
            "goblin",
            TestEntity {
                name: "Goblin".into(),
                hp: 30,
            },
        );
        store.insert(
            "orc",
            TestEntity {
                name: "Orc".into(),
                hp: 50,
            },
        );
        store.insert(
            "troll",
            TestEntity {
                name: "Troll".into(),
                hp: 100,
            },
        );

        // Iterate over values
        let total_hp: i32 = store.values().map(|e| e.hp).sum();
        assert_eq!(total_hp, 180);

        // Filter alive entities
        let alive_count = store.values().filter(|e| e.hp > 0).count();
        assert_eq!(alive_count, 3);
    }

    #[test]
    fn test_store_retain() {
        let mut store = Store::new();
        store.insert(
            "goblin",
            TestEntity {
                name: "Goblin".into(),
                hp: 30,
            },
        );
        store.insert(
            "orc",
            TestEntity {
                name: "Orc".into(),
                hp: 0,
            },
        );
        store.insert(
            "troll",
            TestEntity {
                name: "Troll".into(),
                hp: 100,
            },
        );

        // Remove dead entities
        store.retain(|_, entity| entity.hp > 0);

        assert_eq!(store.len(), 2);
        assert!(!store.contains_key(&"orc"));
    }

    #[test]
    fn test_entity_store() {
        let mut players: EntityStore<TestEntity> = EntityStore::new();

        players.insert(
            "alice".to_string(),
            TestEntity {
                name: "Alice".into(),
                hp: 100,
            },
        );
        players.insert(
            "bob".to_string(),
            TestEntity {
                name: "Bob".into(),
                hp: 90,
            },
        );

        assert_eq!(players.len(), 2);
        assert_eq!(players.get(&"alice".to_string()).unwrap().hp, 100);
    }

    #[test]
    fn test_from_iterator() {
        let data = vec![
            (
                "goblin",
                TestEntity {
                    name: "Goblin".into(),
                    hp: 30,
                },
            ),
            (
                "orc",
                TestEntity {
                    name: "Orc".into(),
                    hp: 50,
                },
            ),
        ];

        let store: Store<&str, TestEntity> = data.into_iter().collect();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_with_capacity() {
        let store: Store<String, TestEntity> = Store::with_capacity(10);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }
}
