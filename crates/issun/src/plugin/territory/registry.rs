//! Territory registry resource

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry of all territories in the game
///
/// This resource manages all territories and provides methods for
/// querying and modifying them.
///
/// # Example
///
/// ```
/// use issun::plugin::territory::{TerritoryRegistry, Territory};
///
/// let mut registry = TerritoryRegistry::new();
///
/// // Add territories
/// registry.add(Territory::new("nova-harbor", "Nova Harbor"));
/// registry.add(Territory::new("rust-city", "Rust City"));
///
/// // Query
/// assert_eq!(registry.len(), 2);
/// assert!(registry.get(&"nova-harbor".into()).is_some());
///
/// // Adjust control
/// let result = registry.adjust_control(&"nova-harbor".into(), 0.3);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryRegistry {
    territories: HashMap<TerritoryId, Territory>,
}

impl TerritoryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            territories: HashMap::new(),
        }
    }

    /// Add a new territory to the registry
    ///
    /// If a territory with the same ID already exists, it will be replaced.
    pub fn add(&mut self, territory: Territory) {
        self.territories.insert(territory.id.clone(), territory);
    }

    /// Get territory by id (immutable)
    pub fn get(&self, id: &TerritoryId) -> Option<&Territory> {
        self.territories.get(id)
    }

    /// Get mutable territory by id
    pub fn get_mut(&mut self, id: &TerritoryId) -> Option<&mut Territory> {
        self.territories.get_mut(id)
    }

    /// Remove a territory from the registry
    pub fn remove(&mut self, id: &TerritoryId) -> Option<Territory> {
        self.territories.remove(id)
    }

    /// Check if a territory exists
    pub fn contains(&self, id: &TerritoryId) -> bool {
        self.territories.contains_key(id)
    }

    /// Get the number of territories
    pub fn len(&self) -> usize {
        self.territories.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.territories.is_empty()
    }

    /// Iterate over all territories (immutable)
    pub fn iter(&self) -> impl Iterator<Item = &Territory> {
        self.territories.values()
    }

    /// Iterate over all territories (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Territory> {
        self.territories.values_mut()
    }

    /// Adjust control of a territory (clamped to 0.0-1.0)
    ///
    /// Returns `ControlChanged` on success, `TerritoryError` on failure.
    ///
    /// # Arguments
    ///
    /// * `id` - Territory identifier
    /// * `delta` - Change in control (can be negative)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::territory::{TerritoryRegistry, Territory, TerritoryId};
    ///
    /// let mut registry = TerritoryRegistry::new();
    /// registry.add(Territory::new("nova", "Nova Harbor").with_control(0.5));
    ///
    /// let result = registry.adjust_control(&TerritoryId::new("nova"), 0.2);
    /// assert!(result.is_ok());
    ///
    /// let change = result.unwrap();
    /// assert_eq!(change.old_control, 0.5);
    /// assert_eq!(change.new_control, 0.7);
    /// assert_eq!(change.delta, 0.2);
    /// ```
    pub fn adjust_control(
        &mut self,
        id: &TerritoryId,
        delta: f32,
    ) -> Result<ControlChanged, TerritoryError> {
        let territory = self
            .territories
            .get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        let old_control = territory.control;

        // Delegate to Service for pure calculation logic
        let (new_control, actual_delta) =
            super::service::TerritoryService::calculate_control_change(old_control, delta);

        territory.control = new_control;

        Ok(ControlChanged {
            id: id.clone(),
            old_control,
            new_control,
            delta: actual_delta,
        })
    }

    /// Set control of a territory directly (clamped to 0.0-1.0)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::territory::{TerritoryRegistry, Territory, TerritoryId};
    ///
    /// let mut registry = TerritoryRegistry::new();
    /// registry.add(Territory::new("nova", "Nova Harbor"));
    ///
    /// let result = registry.set_control(&TerritoryId::new("nova"), 0.8);
    /// assert!(result.is_ok());
    ///
    /// let territory = registry.get(&TerritoryId::new("nova")).unwrap();
    /// assert_eq!(territory.control, 0.8);
    /// ```
    pub fn set_control(
        &mut self,
        id: &TerritoryId,
        control: f32,
    ) -> Result<ControlChanged, TerritoryError> {
        let territory = self
            .territories
            .get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        let old_control = territory.control;
        territory.control = control.clamp(0.0, 1.0);

        Ok(ControlChanged {
            id: id.clone(),
            old_control,
            new_control: territory.control,
            delta: territory.control - old_control,
        })
    }

    /// Develop territory (increase development level by 1)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::territory::{TerritoryRegistry, Territory, TerritoryId};
    ///
    /// let mut registry = TerritoryRegistry::new();
    /// registry.add(Territory::new("nova", "Nova Harbor").with_development(2));
    ///
    /// let result = registry.develop(&TerritoryId::new("nova"));
    /// assert!(result.is_ok());
    ///
    /// let developed = result.unwrap();
    /// assert_eq!(developed.old_level, 2);
    /// assert_eq!(developed.new_level, 3);
    /// ```
    pub fn develop(&mut self, id: &TerritoryId) -> Result<Developed, TerritoryError> {
        let territory = self
            .territories
            .get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        let old_level = territory.development_level;
        territory.development_level += 1;

        Ok(Developed {
            id: id.clone(),
            old_level,
            new_level: territory.development_level,
        })
    }

    /// Set development level directly
    pub fn set_development(
        &mut self,
        id: &TerritoryId,
        level: u32,
    ) -> Result<Developed, TerritoryError> {
        let territory = self
            .territories
            .get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        let old_level = territory.development_level;
        territory.development_level = level;

        Ok(Developed {
            id: id.clone(),
            old_level,
            new_level: territory.development_level,
        })
    }

    /// Update effects for a territory
    pub fn set_effects(
        &mut self,
        id: &TerritoryId,
        effects: TerritoryEffects,
    ) -> Result<(), TerritoryError> {
        let territory = self
            .territories
            .get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        territory.effects = effects;
        Ok(())
    }
}

impl Default for TerritoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = TerritoryRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_add_and_get() {
        let mut registry = TerritoryRegistry::new();

        let territory = Territory::new("nova", "Nova Harbor");
        registry.add(territory);

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let retrieved = registry.get(&TerritoryId::new("nova"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Nova Harbor");
    }

    #[test]
    fn test_registry_contains() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor"));

        assert!(registry.contains(&TerritoryId::new("nova")));
        assert!(!registry.contains(&TerritoryId::new("rust")));
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor"));

        assert_eq!(registry.len(), 1);

        let removed = registry.remove(&TerritoryId::new("nova"));
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);

        let removed_again = registry.remove(&TerritoryId::new("nova"));
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_registry_iter() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor"));
        registry.add(Territory::new("rust", "Rust City"));

        let count = registry.iter().count();
        assert_eq!(count, 2);

        let names: Vec<_> = registry.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Nova Harbor"));
        assert!(names.contains(&"Rust City"));
    }

    #[test]
    fn test_adjust_control() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor").with_control(0.5));

        // Increase control
        let result = registry.adjust_control(&TerritoryId::new("nova"), 0.2);
        assert!(result.is_ok());

        let change = result.unwrap();
        assert_eq!(change.old_control, 0.5);
        assert!((change.new_control - 0.7).abs() < 0.001);
        assert!((change.delta - 0.2).abs() < 0.001);

        // Check clamping (upper bound)
        let result = registry.adjust_control(&TerritoryId::new("nova"), 0.5);
        assert!(result.is_ok());
        let change = result.unwrap();
        assert_eq!(change.new_control, 1.0);
        assert!((change.delta - 0.3).abs() < 0.001); // Only increased by 0.3 to reach 1.0

        // Decrease control
        let result = registry.adjust_control(&TerritoryId::new("nova"), -0.3);
        assert!(result.is_ok());
        let change = result.unwrap();
        assert!((change.new_control - 0.7).abs() < 0.001);
        assert!((change.delta - (-0.3)).abs() < 0.001);

        // Check clamping (lower bound)
        let result = registry.adjust_control(&TerritoryId::new("nova"), -1.0);
        assert!(result.is_ok());
        let change = result.unwrap();
        assert_eq!(change.new_control, 0.0);
        assert!((change.delta - (-0.7)).abs() < 0.001); // Only decreased by 0.7 to reach 0.0
    }

    #[test]
    fn test_adjust_control_not_found() {
        let mut registry = TerritoryRegistry::new();

        let result = registry.adjust_control(&TerritoryId::new("nonexistent"), 0.1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TerritoryError::NotFound);
    }

    #[test]
    fn test_set_control() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor"));

        let result = registry.set_control(&TerritoryId::new("nova"), 0.8);
        assert!(result.is_ok());

        let territory = registry.get(&TerritoryId::new("nova")).unwrap();
        assert_eq!(territory.control, 0.8);

        // Test clamping
        let result = registry.set_control(&TerritoryId::new("nova"), 1.5);
        assert!(result.is_ok());
        let territory = registry.get(&TerritoryId::new("nova")).unwrap();
        assert_eq!(territory.control, 1.0);
    }

    #[test]
    fn test_develop() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor").with_development(2));

        let result = registry.develop(&TerritoryId::new("nova"));
        assert!(result.is_ok());

        let developed = result.unwrap();
        assert_eq!(developed.old_level, 2);
        assert_eq!(developed.new_level, 3);

        let territory = registry.get(&TerritoryId::new("nova")).unwrap();
        assert_eq!(territory.development_level, 3);
    }

    #[test]
    fn test_set_development() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor"));

        let result = registry.set_development(&TerritoryId::new("nova"), 5);
        assert!(result.is_ok());

        let territory = registry.get(&TerritoryId::new("nova")).unwrap();
        assert_eq!(territory.development_level, 5);
    }

    #[test]
    fn test_set_effects() {
        let mut registry = TerritoryRegistry::new();
        registry.add(Territory::new("nova", "Nova Harbor"));

        let effects = TerritoryEffects::default()
            .with_income_multiplier(1.5)
            .with_cost_multiplier(0.8);

        let result = registry.set_effects(&TerritoryId::new("nova"), effects.clone());
        assert!(result.is_ok());

        let territory = registry.get(&TerritoryId::new("nova")).unwrap();
        assert_eq!(territory.effects.income_multiplier, 1.5);
        assert_eq!(territory.effects.cost_multiplier, 0.8);
    }
}
