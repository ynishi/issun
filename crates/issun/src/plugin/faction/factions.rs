//! Faction definitions (ReadOnly asset)

use super::types::*;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection of faction definitions (ReadOnly)
///
/// This is an asset loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factions {
    factions: HashMap<FactionId, Faction>,
}

impl Resource for Factions {}

impl Factions {
    /// Create a new empty factions collection
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
        }
    }

    /// Add a faction definition
    pub fn add(&mut self, faction: Faction) {
        self.factions.insert(faction.id.clone(), faction);
    }

    /// Get a faction by id
    pub fn get(&self, id: &FactionId) -> Option<&Faction> {
        self.factions.get(id)
    }

    /// Check if a faction exists
    pub fn contains(&self, id: &FactionId) -> bool {
        self.factions.contains_key(id)
    }

    /// List all factions
    pub fn iter(&self) -> impl Iterator<Item = &Faction> {
        self.factions.values()
    }

    /// Get the number of factions
    pub fn len(&self) -> usize {
        self.factions.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.factions.is_empty()
    }
}

impl Default for Factions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factions_new() {
        let factions = Factions::new();
        assert!(factions.is_empty());
        assert_eq!(factions.len(), 0);
    }

    #[test]
    fn test_add_and_get() {
        let mut factions = Factions::new();
        let faction = Faction::new("crimson", "Crimson Syndicate");
        factions.add(faction);

        assert_eq!(factions.len(), 1);
        assert!(!factions.is_empty());
        assert!(factions.get(&FactionId::new("crimson")).is_some());
        assert_eq!(
            factions.get(&FactionId::new("crimson")).unwrap().name,
            "Crimson Syndicate"
        );
    }

    #[test]
    fn test_contains() {
        let mut factions = Factions::new();
        factions.add(Faction::new("crimson", "Crimson Syndicate"));

        assert!(factions.contains(&FactionId::new("crimson")));
        assert!(!factions.contains(&FactionId::new("azure")));
    }

    #[test]
    fn test_iter() {
        let mut factions = Factions::new();
        factions.add(Faction::new("crimson", "Crimson Syndicate"));
        factions.add(Faction::new("azure", "Azure Collective"));

        let count = factions.iter().count();
        assert_eq!(count, 2);
    }
}
