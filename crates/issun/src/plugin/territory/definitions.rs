//! Territory definitions (read-only asset/config)

use super::types::*;
use issun_macros::Resource as DeriveResource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Territory definitions (read-only)
///
/// Contains all territory definitions loaded from assets/config.
/// These are immutable during gameplay.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::territory::{TerritoryDefinitions, Territory};
///
/// let mut definitions = TerritoryDefinitions::new();
/// definitions.add(Territory::new("nova", "Nova Harbor"));
/// definitions.add(Territory::new("rust-city", "Rust City"));
///
/// // Query
/// if let Some(territory) = definitions.get(&"nova".into()) {
///     println!("Found: {}", territory.name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, DeriveResource)]
pub struct TerritoryDefinitions {
    territories: HashMap<TerritoryId, Territory>,
}

impl TerritoryDefinitions {
    /// Create a new empty definitions
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

impl Default for TerritoryDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definitions_new() {
        let definitions = TerritoryDefinitions::new();
        assert_eq!(definitions.len(), 0);
        assert!(definitions.is_empty());
    }

    #[test]
    fn test_add_and_get() {
        let mut definitions = TerritoryDefinitions::new();
        let territory = Territory::new("nova", "Nova Harbor");
        definitions.add(territory);

        assert_eq!(definitions.len(), 1);
        assert!(!definitions.is_empty());

        let retrieved = definitions.get(&TerritoryId::new("nova"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Nova Harbor");
    }

    #[test]
    fn test_contains() {
        let mut definitions = TerritoryDefinitions::new();
        definitions.add(Territory::new("nova", "Nova Harbor"));

        assert!(definitions.contains(&TerritoryId::new("nova")));
        assert!(!definitions.contains(&TerritoryId::new("rust")));
    }

    #[test]
    fn test_iter() {
        let mut definitions = TerritoryDefinitions::new();
        definitions.add(Territory::new("nova", "Nova Harbor"));
        definitions.add(Territory::new("rust", "Rust City"));

        let count = definitions.iter().count();
        assert_eq!(count, 2);

        let names: Vec<_> = definitions.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Nova Harbor"));
        assert!(names.contains(&"Rust City"));
    }

    #[test]
    fn test_query() {
        let mut definitions = TerritoryDefinitions::new();
        definitions.add(Territory::new("nova", "Nova Harbor").with_control(0.8));
        definitions.add(Territory::new("rust", "Rust City").with_control(0.3));

        // Query territories with high initial control
        let high_control = definitions.query(|t| t.control > 0.5);
        assert_eq!(high_control.len(), 1);
        assert_eq!(high_control[0].id.as_str(), "nova");
    }
}
