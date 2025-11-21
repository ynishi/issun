//! Territory collection (read-only resource)

use super::types::*;
use issun_macros::Resource as DeriveResource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection of all territory definitions (read-only)
///
/// Contains all territory definitions loaded from assets/config.
/// These are immutable during gameplay.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::territory::{Territories, Territory};
///
/// let mut territories = Territories::new();
/// territories.add(Territory::new("nova", "Nova Harbor"));
/// territories.add(Territory::new("rust-city", "Rust City"));
///
/// // Query
/// if let Some(territory) = territories.get(&"nova".into()) {
///     println!("Found: {}", territory.name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, DeriveResource)]
pub struct Territories {
    territories: HashMap<TerritoryId, Territory>,
}

impl Territories {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            territories: HashMap::new(),
        }
    }

    /// Add a territory definition
    pub fn add(&mut self, territory: Territory) {
        self.territories.insert(territory.id.clone(), territory);
    }

    /// Get territory by id
    pub fn get(&self, id: &TerritoryId) -> Option<&Territory> {
        self.territories.get(id)
    }

    /// Check if a territory exists
    pub fn contains(&self, id: &TerritoryId) -> bool {
        self.territories.contains_key(id)
    }

    /// Get the number of territories
    pub fn len(&self) -> usize {
        self.territories.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.territories.is_empty()
    }

    /// Iterate over all territories
    pub fn iter(&self) -> impl Iterator<Item = &Territory> {
        self.territories.values()
    }

    /// Query territories by predicate
    pub fn query<F>(&self, predicate: F) -> Vec<&Territory>
    where
        F: Fn(&Territory) -> bool,
    {
        self.territories.values().filter(|t| predicate(t)).collect()
    }
}

impl Default for Territories {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_territories_new() {
        let territories = Territories::new();
        assert_eq!(territories.len(), 0);
        assert!(territories.is_empty());
    }

    #[test]
    fn test_add_and_get() {
        let mut territories = Territories::new();
        let territory = Territory::new("nova", "Nova Harbor");
        territories.add(territory);

        assert_eq!(territories.len(), 1);
        assert!(!territories.is_empty());

        let retrieved = territories.get(&TerritoryId::new("nova"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Nova Harbor");
    }

    #[test]
    fn test_contains() {
        let mut territories = Territories::new();
        territories.add(Territory::new("nova", "Nova Harbor"));

        assert!(territories.contains(&TerritoryId::new("nova")));
        assert!(!territories.contains(&TerritoryId::new("rust")));
    }

    #[test]
    fn test_iter() {
        let mut territories = Territories::new();
        territories.add(Territory::new("nova", "Nova Harbor"));
        territories.add(Territory::new("rust", "Rust City"));

        let count = territories.iter().count();
        assert_eq!(count, 2);

        let names: Vec<_> = territories.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Nova Harbor"));
        assert!(names.contains(&"Rust City"));
    }

    #[test]
    fn test_query() {
        let mut territories = Territories::new();
        territories.add(Territory::new("nova", "Nova Harbor"));
        territories.add(Territory::new("rust", "Rust City"));

        // Query by name
        let nova_only = territories.query(|t| t.name == "Nova Harbor");
        assert_eq!(nova_only.len(), 1);
        assert_eq!(nova_only[0].id.as_str(), "nova");
    }
}
