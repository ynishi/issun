//! Territory runtime state (mutable, save/load target)

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Territory runtime state (mutable)
///
/// Contains runtime state that changes during gameplay.
/// This is a save/load target.
///
/// # Design
///
/// - **TerritoryDefinitions**: Territory definitions (id, name) - ReadOnly
/// - **TerritoryState**: Runtime state (control, development) - Mutable
///
/// # Example
///
/// ```ignore
/// use issun::plugin::territory::TerritoryState;
///
/// let mut state = TerritoryState::new();
///
/// // Initialize territories
/// state.initialize(&"nova".into());
/// state.initialize(&"rust-city".into());
///
/// // Update control
/// state.set_control(&"nova".into(), 0.5);
/// state.set_development(&"nova".into(), 3);
///
/// // Query
/// let control = state.get_control(&"nova".into());
/// assert_eq!(control, Some(0.5));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryState {
    /// Control values: 0.0 (no control) to 1.0 (full control)
    control: HashMap<TerritoryId, f32>,

    /// Development levels: 0 (undeveloped) to N
    development: HashMap<TerritoryId, u32>,

    /// Territory effects
    effects: HashMap<TerritoryId, TerritoryEffects>,
}

impl State for TerritoryState {}

impl TerritoryState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            control: HashMap::new(),
            development: HashMap::new(),
            effects: HashMap::new(),
        }
    }

    /// Initialize a territory with default values
    ///
    /// This should be called when a territory is added to the game.
    pub fn initialize(&mut self, id: &TerritoryId) {
        self.control.insert(id.clone(), 0.0);
        self.development.insert(id.clone(), 0);
        self.effects
            .insert(id.clone(), TerritoryEffects::default());
    }

    /// Check if a territory is initialized
    pub fn contains(&self, id: &TerritoryId) -> bool {
        self.control.contains_key(id)
    }

    // ========================================
    // Control Management
    // ========================================

    /// Get control value
    pub fn get_control(&self, id: &TerritoryId) -> Option<f32> {
        self.control.get(id).copied()
    }

    /// Set control value (clamped to 0.0-1.0)
    pub fn set_control(&mut self, id: &TerritoryId, control: f32) -> Option<ControlChanged> {
        let old_control = *self.control.get(id)?;
        let new_control = control.clamp(0.0, 1.0);
        self.control.insert(id.clone(), new_control);

        Some(ControlChanged {
            id: id.clone(),
            old_control,
            new_control,
            delta: new_control - old_control,
        })
    }

    /// Adjust control by delta (clamped to 0.0-1.0)
    ///
    /// Returns actual delta applied (may differ from requested if clamping occurred).
    pub fn adjust_control(&mut self, id: &TerritoryId, delta: f32) -> Option<ControlChanged> {
        let old_control = *self.control.get(id)?;

        // Delegate to Service for pure calculation logic
        let (new_control, actual_delta) =
            super::service::TerritoryService::calculate_control_change(old_control, delta);

        self.control.insert(id.clone(), new_control);

        Some(ControlChanged {
            id: id.clone(),
            old_control,
            new_control,
            delta: actual_delta,
        })
    }

    // ========================================
    // Development Management
    // ========================================

    /// Get development level
    pub fn get_development(&self, id: &TerritoryId) -> Option<u32> {
        self.development.get(id).copied()
    }

    /// Set development level
    pub fn set_development(&mut self, id: &TerritoryId, level: u32) -> Option<Developed> {
        let old_level = *self.development.get(id)?;
        self.development.insert(id.clone(), level);

        Some(Developed {
            id: id.clone(),
            old_level,
            new_level: level,
        })
    }

    /// Develop territory (increase development level by 1)
    pub fn develop(&mut self, id: &TerritoryId) -> Option<Developed> {
        let old_level = *self.development.get(id)?;
        let new_level = old_level + 1;
        self.development.insert(id.clone(), new_level);

        Some(Developed {
            id: id.clone(),
            old_level,
            new_level,
        })
    }

    // ========================================
    // Effects Management
    // ========================================

    /// Get territory effects
    pub fn get_effects(&self, id: &TerritoryId) -> Option<&TerritoryEffects> {
        self.effects.get(id)
    }

    /// Set territory effects
    pub fn set_effects(&mut self, id: &TerritoryId, effects: TerritoryEffects) -> bool {
        if !self.effects.contains_key(id) {
            return false;
        }
        self.effects.insert(id.clone(), effects);
        true
    }

    // ========================================
    // Queries
    // ========================================

    /// Get all territory IDs
    pub fn territory_ids(&self) -> impl Iterator<Item = &TerritoryId> {
        self.control.keys()
    }

    /// Get territories with control above threshold
    pub fn controlled_territories(&self, threshold: f32) -> Vec<TerritoryId> {
        self.control
            .iter()
            .filter(|(_, &control)| control >= threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get territories at or above development level
    pub fn developed_territories(&self, min_level: u32) -> Vec<TerritoryId> {
        self.development
            .iter()
            .filter(|(_, &level)| level >= min_level)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for TerritoryState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = TerritoryState::new();
        assert!(!state.contains(&TerritoryId::new("nova")));
    }

    #[test]
    fn test_initialize() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));

        assert!(state.contains(&TerritoryId::new("nova")));
        assert_eq!(state.get_control(&TerritoryId::new("nova")), Some(0.0));
        assert_eq!(state.get_development(&TerritoryId::new("nova")), Some(0));
    }

    #[test]
    fn test_set_control() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));

        let change = state.set_control(&TerritoryId::new("nova"), 0.5);
        assert!(change.is_some());

        let change = change.unwrap();
        assert_eq!(change.old_control, 0.0);
        assert_eq!(change.new_control, 0.5);
        assert_eq!(change.delta, 0.5);

        assert_eq!(state.get_control(&TerritoryId::new("nova")), Some(0.5));
    }

    #[test]
    fn test_set_control_clamping() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));

        // Test upper bound
        state.set_control(&TerritoryId::new("nova"), 1.5);
        assert_eq!(state.get_control(&TerritoryId::new("nova")), Some(1.0));

        // Test lower bound
        state.set_control(&TerritoryId::new("nova"), -0.5);
        assert_eq!(state.get_control(&TerritoryId::new("nova")), Some(0.0));
    }

    #[test]
    fn test_adjust_control() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));
        state.set_control(&TerritoryId::new("nova"), 0.5);

        // Increase control
        let change = state.adjust_control(&TerritoryId::new("nova"), 0.2);
        assert!(change.is_some());
        let change = change.unwrap();
        assert_eq!(change.old_control, 0.5);
        assert!((change.new_control - 0.7).abs() < 0.001);
        assert!((change.delta - 0.2).abs() < 0.001);

        // Check clamping (upper bound)
        let change = state.adjust_control(&TerritoryId::new("nova"), 0.5);
        assert!(change.is_some());
        let change = change.unwrap();
        assert_eq!(change.new_control, 1.0);
        assert!((change.delta - 0.3).abs() < 0.001); // Only increased by 0.3 to reach 1.0
    }

    #[test]
    fn test_develop() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));

        let dev = state.develop(&TerritoryId::new("nova"));
        assert!(dev.is_some());

        let dev = dev.unwrap();
        assert_eq!(dev.old_level, 0);
        assert_eq!(dev.new_level, 1);

        assert_eq!(state.get_development(&TerritoryId::new("nova")), Some(1));
    }

    #[test]
    fn test_set_effects() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));

        let effects = TerritoryEffects::default()
            .with_income_multiplier(1.5)
            .with_cost_multiplier(0.8);

        assert!(state.set_effects(&TerritoryId::new("nova"), effects.clone()));

        let retrieved = state.get_effects(&TerritoryId::new("nova"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().income_multiplier, 1.5);
        assert_eq!(retrieved.unwrap().cost_multiplier, 0.8);
    }

    #[test]
    fn test_controlled_territories() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));
        state.initialize(&TerritoryId::new("rust"));
        state.initialize(&TerritoryId::new("vapor"));

        state.set_control(&TerritoryId::new("nova"), 0.8);
        state.set_control(&TerritoryId::new("rust"), 0.3);
        state.set_control(&TerritoryId::new("vapor"), 1.0);

        let controlled = state.controlled_territories(0.5);
        assert_eq!(controlled.len(), 2);
    }

    #[test]
    fn test_developed_territories() {
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));
        state.initialize(&TerritoryId::new("rust"));

        state.set_development(&TerritoryId::new("nova"), 5);
        state.set_development(&TerritoryId::new("rust"), 2);

        let developed = state.developed_territories(3);
        assert_eq!(developed.len(), 1);
        assert_eq!(developed[0].as_str(), "nova");
    }
}
