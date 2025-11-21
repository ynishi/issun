//! Territory service for domain logic
//!
//! Provides pure functions for territory control and development calculations.
//! All functions are stateless and can be used independently.

use super::state::TerritoryState;
use super::territories::Territories;
use super::types::TerritoryId;
use crate::context::ResourceContext;

/// Territory service providing pure territory calculation logic
///
/// This service handles stateless calculations for territory operations.
/// It follows Domain-Driven Design principles - territory logic as a service.
///
/// # Design Philosophy
///
/// - **Stateless**: All functions are pure, taking inputs and returning outputs
/// - **Testable**: No dependencies on Registry or Resources
/// - **Reusable**: Can be called from Registry, Hook, or game code directly
///
/// # Example
///
/// ```ignore
/// use issun::plugin::territory::TerritoryService;
///
/// // Calculate control change with clamping
/// let (new_control, actual_delta) = TerritoryService::calculate_control_change(
///     0.7, // current
///     0.5, // delta
/// );
/// assert_eq!(new_control, 1.0); // Clamped to 1.0
/// assert_eq!(actual_delta, 0.3); // Only increased by 0.3
/// ```
#[derive(Debug, Clone, Default)]
pub struct TerritoryService;

impl TerritoryService {
    /// Create a new territory service
    pub fn new() -> Self {
        Self
    }

    /// Calculate control change with clamping
    ///
    /// # Formula
    ///
    /// ```text
    /// new_control = clamp(current_control + delta, 0.0, 1.0)
    /// actual_delta = new_control - current_control
    /// ```
    ///
    /// # Arguments
    ///
    /// * `current_control` - Current control value (0.0-1.0)
    /// * `delta` - Change amount (can be negative)
    ///
    /// # Returns
    ///
    /// Tuple of `(new_control, actual_delta)` where:
    /// - `new_control`: New control value (clamped to 0.0-1.0)
    /// - `actual_delta`: Actual change applied (may differ from delta due to clamping)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Normal increase
    /// let (new, delta) = TerritoryService::calculate_control_change(0.5, 0.3);
    /// assert_eq!(new, 0.8);
    /// assert_eq!(delta, 0.3);
    ///
    /// // Clamped to 1.0
    /// let (new, delta) = TerritoryService::calculate_control_change(0.7, 0.5);
    /// assert_eq!(new, 1.0);
    /// assert_eq!(delta, 0.3); // Only increased by 0.3
    ///
    /// // Decrease
    /// let (new, delta) = TerritoryService::calculate_control_change(0.5, -0.2);
    /// assert_eq!(new, 0.3);
    /// assert_eq!(delta, -0.2);
    ///
    /// // Clamped to 0.0
    /// let (new, delta) = TerritoryService::calculate_control_change(0.3, -0.5);
    /// assert_eq!(new, 0.0);
    /// assert_eq!(delta, -0.3);
    /// ```
    pub fn calculate_control_change(current_control: f32, delta: f32) -> (f32, f32) {
        let new_control = (current_control + delta).clamp(0.0, 1.0);
        let actual_delta = new_control - current_control;
        (new_control, actual_delta)
    }

    /// Calculate development cost
    ///
    /// Cost increases exponentially with level.
    ///
    /// # Formula
    ///
    /// ```text
    /// cost = base_cost * (1.5 ^ current_level) * cost_multiplier
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_cost` - Base development cost for level 0→1
    /// * `current_level` - Current development level
    /// * `cost_multiplier` - Multiplier from game state (e.g., economic bonuses)
    ///
    /// # Returns
    ///
    /// Development cost for next level
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Level 0→1: Base cost
    /// let cost = TerritoryService::calculate_development_cost(1000, 0, 1.0);
    /// assert_eq!(cost, 1000);
    ///
    /// // Level 1→2: 1.5x base cost
    /// let cost = TerritoryService::calculate_development_cost(1000, 1, 1.0);
    /// assert_eq!(cost, 1500);
    ///
    /// // Level 2→3: 2.25x base cost
    /// let cost = TerritoryService::calculate_development_cost(1000, 2, 1.0);
    /// assert_eq!(cost, 2250);
    ///
    /// // With cost multiplier (50% discount)
    /// let cost = TerritoryService::calculate_development_cost(1000, 2, 0.5);
    /// assert_eq!(cost, 1125);
    /// ```
    pub fn calculate_development_cost(
        base_cost: i64,
        current_level: u32,
        cost_multiplier: f32,
    ) -> i64 {
        let level_scaling = 1.5_f32.powi(current_level as i32);
        let final_cost = base_cost as f32 * level_scaling * cost_multiplier;
        final_cost.max(0.0).round() as i64
    }

    /// Calculate adjacency bonus
    ///
    /// Bonus based on proportion of controlled neighboring territories.
    ///
    /// # Formula
    ///
    /// ```text
    /// bonus = (controlled_neighbors / total_neighbors) * max_bonus
    /// ```
    ///
    /// # Arguments
    ///
    /// * `controlled_neighbors` - Number of neighboring territories under control
    /// * `total_neighbors` - Total number of neighboring territories
    /// * `max_bonus` - Maximum bonus when all neighbors are controlled
    ///
    /// # Returns
    ///
    /// Adjacency bonus (0.0 to max_bonus)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // No neighbors controlled
    /// let bonus = TerritoryService::calculate_adjacency_bonus(0, 4, 0.5);
    /// assert_eq!(bonus, 0.0);
    ///
    /// // Half controlled
    /// let bonus = TerritoryService::calculate_adjacency_bonus(2, 4, 0.5);
    /// assert_eq!(bonus, 0.25); // 50% * 0.5
    ///
    /// // All controlled
    /// let bonus = TerritoryService::calculate_adjacency_bonus(4, 4, 0.5);
    /// assert_eq!(bonus, 0.5);
    ///
    /// // No neighbors (isolated territory)
    /// let bonus = TerritoryService::calculate_adjacency_bonus(0, 0, 0.5);
    /// assert_eq!(bonus, 0.0);
    /// ```
    pub fn calculate_adjacency_bonus(
        controlled_neighbors: usize,
        total_neighbors: usize,
        max_bonus: f32,
    ) -> f32 {
        if total_neighbors == 0 {
            return 0.0; // No neighbors = no bonus
        }

        let ratio = controlled_neighbors as f32 / total_neighbors as f32;
        ratio * max_bonus
    }

    // ========================================
    // ResourceContext Helpers (for Hooks)
    // ========================================

    /// Get territory control value from ResourceContext
    ///
    /// This is a convenience helper for Hooks to easily access territory control
    /// without manually combining Territories and TerritoryState.
    ///
    /// # Arguments
    ///
    /// * `territory_id` - ID of the territory
    /// * `resources` - Resource context containing TerritoryState
    ///
    /// # Returns
    ///
    /// Control value (0.0-1.0) or 0.0 if not found
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In a Hook
    /// let control = TerritoryService::get_control(&territory_id, resources).await;
    /// let income = (base_income as f32 * control) as i64;
    /// ```
    pub async fn get_control(territory_id: &TerritoryId, resources: &ResourceContext) -> f32 {
        if let Some(state) = resources.get::<TerritoryState>().await {
            state.get_control(territory_id)
        } else {
            0.0
        }
    }

    /// Get territory development level from ResourceContext
    ///
    /// # Arguments
    ///
    /// * `territory_id` - ID of the territory
    /// * `resources` - Resource context containing TerritoryState
    ///
    /// # Returns
    ///
    /// Development level or 0 if not found
    pub async fn get_development_level(
        territory_id: &TerritoryId,
        resources: &ResourceContext,
    ) -> u32 {
        if let Some(state) = resources.get::<TerritoryState>().await {
            state.get_development_level(territory_id)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_control_change() {
        // Normal increase
        let (new, delta) = TerritoryService::calculate_control_change(0.5, 0.3);
        assert!((new - 0.8).abs() < 0.001);
        assert!((delta - 0.3).abs() < 0.001);

        // Clamped to 1.0
        let (new, delta) = TerritoryService::calculate_control_change(0.7, 0.5);
        assert_eq!(new, 1.0);
        assert!((delta - 0.3).abs() < 0.001);

        // Normal decrease
        let (new, delta) = TerritoryService::calculate_control_change(0.5, -0.2);
        assert!((new - 0.3).abs() < 0.001);
        assert!((delta - (-0.2)).abs() < 0.001);

        // Clamped to 0.0
        let (new, delta) = TerritoryService::calculate_control_change(0.3, -0.5);
        assert_eq!(new, 0.0);
        assert!((delta - (-0.3)).abs() < 0.001);

        // No change
        let (new, delta) = TerritoryService::calculate_control_change(0.5, 0.0);
        assert_eq!(new, 0.5);
        assert_eq!(delta, 0.0);

        // Already at 1.0
        let (new, delta) = TerritoryService::calculate_control_change(1.0, 0.1);
        assert_eq!(new, 1.0);
        assert_eq!(delta, 0.0);

        // Already at 0.0
        let (new, delta) = TerritoryService::calculate_control_change(0.0, -0.1);
        assert_eq!(new, 0.0);
        assert_eq!(delta, 0.0);
    }

    #[test]
    fn test_calculate_development_cost() {
        // Level 0→1
        let cost = TerritoryService::calculate_development_cost(1000, 0, 1.0);
        assert_eq!(cost, 1000);

        // Level 1→2
        let cost = TerritoryService::calculate_development_cost(1000, 1, 1.0);
        assert_eq!(cost, 1500);

        // Level 2→3
        let cost = TerritoryService::calculate_development_cost(1000, 2, 1.0);
        assert_eq!(cost, 2250);

        // Level 3→4
        let cost = TerritoryService::calculate_development_cost(1000, 3, 1.0);
        assert_eq!(cost, 3375);

        // With cost multiplier (50% discount)
        let cost = TerritoryService::calculate_development_cost(1000, 2, 0.5);
        assert_eq!(cost, 1125);

        // With cost multiplier (2x)
        let cost = TerritoryService::calculate_development_cost(1000, 1, 2.0);
        assert_eq!(cost, 3000);

        // Zero base cost
        let cost = TerritoryService::calculate_development_cost(0, 5, 1.0);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_calculate_adjacency_bonus() {
        // No neighbors controlled
        let bonus = TerritoryService::calculate_adjacency_bonus(0, 4, 0.5);
        assert!((bonus - 0.0).abs() < 0.001);

        // Half controlled
        let bonus = TerritoryService::calculate_adjacency_bonus(2, 4, 0.5);
        assert!((bonus - 0.25).abs() < 0.001); // 50% * 0.5

        // All controlled
        let bonus = TerritoryService::calculate_adjacency_bonus(4, 4, 0.5);
        assert!((bonus - 0.5).abs() < 0.001);

        // One neighbor controlled
        let bonus = TerritoryService::calculate_adjacency_bonus(1, 3, 0.6);
        assert!((bonus - 0.2).abs() < 0.001); // (1/3) * 0.6

        // No neighbors (isolated territory)
        let bonus = TerritoryService::calculate_adjacency_bonus(0, 0, 0.5);
        assert_eq!(bonus, 0.0);

        // Different max bonus
        let bonus = TerritoryService::calculate_adjacency_bonus(3, 3, 1.0);
        assert!((bonus - 1.0).abs() < 0.001);
    }
}
