//! Faction service for domain logic
//!
//! Provides pure functions for faction operations and calculations.
//! All functions are stateless and can be used independently.

use super::types::Outcome;
use rand::Rng;

/// Faction service providing pure faction calculation logic
///
/// This service handles stateless calculations for faction operations.
/// It follows Domain-Driven Design principles - faction logic as a service.
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
/// use issun::plugin::faction::{FactionService, Faction, Operation, Outcome};
/// use rand::thread_rng;
///
/// let faction = Faction::new("crimson", "Crimson Syndicate");
/// let operation = Operation::new("op-001", faction.id.clone(), "Sabotage Mission");
///
/// // Calculate operation cost
/// let base_cost = 1000;
/// let cost = FactionService::calculate_operation_cost(base_cost, 1.2, &[0.9]);
/// assert_eq!(cost, 1080); // 1000 * 1.2 * 0.9
///
/// // Calculate success rate
/// let faction_power = 100.0;
/// let difficulty = 50.0;
/// let success_rate = FactionService::calculate_success_rate(faction_power, difficulty, &[1.1]);
/// assert_eq!(success_rate, 0.715); // (100 / 50) * 1.1 / 4.0, clamped to 0.0-1.0
///
/// // Generate outcome
/// let mut rng = thread_rng();
/// let outcome = FactionService::generate_outcome(
///     operation.id.clone(),
///     success_rate,
///     &mut rng,
/// );
/// ```
#[derive(Debug, Clone, Default)]
pub struct FactionService;

impl FactionService {
    /// Create a new faction service
    pub fn new() -> Self {
        Self
    }

    /// Calculate operation cost
    ///
    /// Applies base cost, cost multiplier, and optional modifiers.
    ///
    /// # Formula
    ///
    /// ```text
    /// cost = base_cost * cost_multiplier * (modifier1 * modifier2 * ...)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_cost` - Base cost of the operation
    /// * `cost_multiplier` - Multiplier from faction/policy effects
    /// * `modifiers` - Additional cost modifiers (e.g., from policies, terrain)
    ///
    /// # Returns
    ///
    /// Final operation cost
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Base cost 1000, multiplier 1.2, modifier 0.9 (10% discount)
    /// let cost = FactionService::calculate_operation_cost(1000, 1.2, &[0.9]);
    /// assert_eq!(cost, 1080); // 1000 * 1.2 * 0.9
    /// ```
    pub fn calculate_operation_cost(
        base_cost: i64,
        cost_multiplier: f32,
        modifiers: &[f32],
    ) -> i64 {
        let modifier_product: f32 = modifiers.iter().product();
        let final_cost = base_cost as f32 * cost_multiplier * modifier_product;
        final_cost.max(0.0).round() as i64
    }

    /// Calculate operation success rate
    ///
    /// Determines success probability based on faction power and operation difficulty.
    ///
    /// # Formula
    ///
    /// ```text
    /// base_rate = (faction_power / difficulty) / 4.0  // Normalized to 0.0-1.0 range
    /// final_rate = base_rate * (modifier1 * modifier2 * ...)
    /// final_rate = clamp(final_rate, 0.0, 1.0)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `faction_power` - Faction's power/strength metric
    /// * `difficulty` - Operation difficulty metric
    /// * `modifiers` - Success rate modifiers (e.g., from policies, equipment)
    ///
    /// # Returns
    ///
    /// Success rate between 0.0 (impossible) and 1.0 (guaranteed)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Equal power and difficulty = 50% base success
    /// let rate = FactionService::calculate_success_rate(100.0, 100.0, &[]);
    /// assert_eq!(rate, 0.25); // (100 / 100) / 4.0 = 0.25
    ///
    /// // Twice the power = 50% base success
    /// let rate = FactionService::calculate_success_rate(200.0, 100.0, &[]);
    /// assert_eq!(rate, 0.5); // (200 / 100) / 4.0 = 0.5
    ///
    /// // With 1.5x modifier
    /// let rate = FactionService::calculate_success_rate(200.0, 100.0, &[1.5]);
    /// assert_eq!(rate, 0.75); // (200 / 100) / 4.0 * 1.5 = 0.75
    ///
    /// // Clamped to 1.0
    /// let rate = FactionService::calculate_success_rate(400.0, 100.0, &[1.5]);
    /// assert_eq!(rate, 1.0); // Clamped
    /// ```
    pub fn calculate_success_rate(faction_power: f32, difficulty: f32, modifiers: &[f32]) -> f32 {
        if difficulty <= 0.0 {
            return 1.0; // No difficulty = guaranteed success
        }

        let power_ratio = faction_power / difficulty;
        let base_rate = power_ratio / 4.0; // Normalize to 0.0-1.0 range (4x power = 100%)

        let modifier_product: f32 = modifiers.iter().product();
        let final_rate = base_rate * modifier_product;

        final_rate.clamp(0.0, 1.0)
    }

    /// Generate operation outcome
    ///
    /// Uses success rate to randomly determine success/failure.
    ///
    /// # Arguments
    ///
    /// * `operation_id` - Operation identifier
    /// * `success_rate` - Success probability (0.0-1.0)
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// Outcome with success/failure determined by random roll
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rand::thread_rng;
    ///
    /// let mut rng = thread_rng();
    ///
    /// // 80% success rate
    /// let outcome = FactionService::generate_outcome(
    ///     "op-001".into(),
    ///     0.8,
    ///     &mut rng,
    /// );
    /// // outcome.success has 80% chance of being true
    /// ```
    pub fn generate_outcome(
        operation_id: impl Into<String>,
        success_rate: f32,
        rng: &mut impl Rng,
    ) -> Outcome {
        let success = rng.gen_bool(success_rate.clamp(0.0, 1.0) as f64);
        Outcome::new(operation_id, success)
    }

    /// Generate outcome with custom metrics
    ///
    /// Uses success rate to determine success/failure, and applies appropriate metrics.
    ///
    /// # Arguments
    ///
    /// * `operation_id` - Operation identifier
    /// * `success_rate` - Success probability (0.0-1.0)
    /// * `rng` - Random number generator
    /// * `success_metrics` - Metrics to apply on success
    /// * `failure_metrics` - Metrics to apply on failure
    ///
    /// # Returns
    ///
    /// Outcome with success/failure and appropriate metrics
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rand::thread_rng;
    /// use std::collections::HashMap;
    ///
    /// let mut rng = thread_rng();
    ///
    /// let success_metrics = HashMap::from([
    ///     ("territory_gained".into(), 1.0),
    ///     ("casualties".into(), 5.0),
    /// ]);
    ///
    /// let failure_metrics = HashMap::from([
    ///     ("casualties".into(), 15.0),
    ///     ("morale_loss".into(), 10.0),
    /// ]);
    ///
    /// let outcome = FactionService::generate_outcome_with_metrics(
    ///     "op-001".into(),
    ///     0.7,
    ///     &mut rng,
    ///     success_metrics,
    ///     failure_metrics,
    /// );
    ///
    /// if outcome.success {
    ///     assert_eq!(outcome.metrics.get("territory_gained"), Some(&1.0));
    /// } else {
    ///     assert_eq!(outcome.metrics.get("morale_loss"), Some(&10.0));
    /// }
    /// ```
    pub fn generate_outcome_with_metrics(
        operation_id: impl Into<String>,
        success_rate: f32,
        rng: &mut impl Rng,
        success_metrics: std::collections::HashMap<String, f32>,
        failure_metrics: std::collections::HashMap<String, f32>,
    ) -> Outcome {
        let success = rng.gen_bool(success_rate.clamp(0.0, 1.0) as f64);

        let mut outcome = Outcome::new(operation_id, success);
        outcome.metrics = if success {
            success_metrics
        } else {
            failure_metrics
        };

        outcome
    }

    /// Calculate relationship change
    ///
    /// Determines how an operation affects faction relationships.
    ///
    /// # Arguments
    ///
    /// * `base_change` - Base relationship change value
    /// * `operation_success` - Whether operation succeeded
    /// * `success_multiplier` - Multiplier applied on success
    /// * `failure_multiplier` - Multiplier applied on failure
    ///
    /// # Returns
    ///
    /// Final relationship change value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Successful operation: +10 base, 1.5x multiplier
    /// let change = FactionService::calculate_relationship_change(10.0, true, 1.5, 0.5);
    /// assert_eq!(change, 15.0); // 10 * 1.5
    ///
    /// // Failed operation: -10 base, 0.5x multiplier (less penalty)
    /// let change = FactionService::calculate_relationship_change(-10.0, false, 1.5, 0.5);
    /// assert_eq!(change, -5.0); // -10 * 0.5
    /// ```
    pub fn calculate_relationship_change(
        base_change: f32,
        operation_success: bool,
        success_multiplier: f32,
        failure_multiplier: f32,
    ) -> f32 {
        if operation_success {
            base_change * success_multiplier
        } else {
            base_change * failure_multiplier
        }
    }

    /// Estimate operation effectiveness
    ///
    /// Calculates expected value of operation based on success rate and potential outcomes.
    ///
    /// # Formula
    ///
    /// ```text
    /// effectiveness = (success_rate * success_value) + ((1 - success_rate) * failure_value)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `success_rate` - Success probability (0.0-1.0)
    /// * `success_value` - Value if successful
    /// * `failure_value` - Value if failed (usually negative)
    ///
    /// # Returns
    ///
    /// Expected value of the operation
    ///
    /// # Example
    ///
    /// ```ignore
    /// // 70% success rate, +100 on success, -30 on failure
    /// let effectiveness = FactionService::estimate_operation_effectiveness(0.7, 100.0, -30.0);
    /// assert_eq!(effectiveness, 61.0); // (0.7 * 100) + (0.3 * -30) = 70 - 9 = 61
    /// ```
    pub fn estimate_operation_effectiveness(
        success_rate: f32,
        success_value: f32,
        failure_value: f32,
    ) -> f32 {
        (success_rate * success_value) + ((1.0 - success_rate) * failure_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_operation_cost() {
        // Base case: no modifiers
        let cost = FactionService::calculate_operation_cost(1000, 1.0, &[]);
        assert_eq!(cost, 1000);

        // With cost multiplier
        let cost = FactionService::calculate_operation_cost(1000, 1.2, &[]);
        assert_eq!(cost, 1200);

        // With modifiers
        let cost = FactionService::calculate_operation_cost(1000, 1.2, &[0.9, 0.8]);
        assert_eq!(cost, 864); // 1000 * 1.2 * 0.9 * 0.8

        // Zero cost
        let cost = FactionService::calculate_operation_cost(0, 1.2, &[0.9]);
        assert_eq!(cost, 0);

        // Negative base cost (should clamp to 0)
        let cost = FactionService::calculate_operation_cost(-100, 1.0, &[]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_calculate_success_rate() {
        // Equal power and difficulty
        let rate = FactionService::calculate_success_rate(100.0, 100.0, &[]);
        assert_eq!(rate, 0.25); // (100 / 100) / 4.0

        // Double power
        let rate = FactionService::calculate_success_rate(200.0, 100.0, &[]);
        assert_eq!(rate, 0.5); // (200 / 100) / 4.0

        // With modifier
        let rate = FactionService::calculate_success_rate(200.0, 100.0, &[1.5]);
        assert_eq!(rate, 0.75); // (200 / 100) / 4.0 * 1.5

        // Clamped to 1.0
        let rate = FactionService::calculate_success_rate(400.0, 100.0, &[1.5]);
        assert_eq!(rate, 1.0);

        // Zero difficulty = guaranteed success
        let rate = FactionService::calculate_success_rate(100.0, 0.0, &[]);
        assert_eq!(rate, 1.0);

        // Very low power
        let rate = FactionService::calculate_success_rate(10.0, 100.0, &[]);
        assert_eq!(rate, 0.025); // (10 / 100) / 4.0
    }

    #[test]
    fn test_generate_outcome() {
        let mut rng = StdRng::seed_from_u64(42);

        // 100% success rate
        let outcome = FactionService::generate_outcome("op-001", 1.0, &mut rng);
        assert!(outcome.success);

        // 0% success rate
        let outcome = FactionService::generate_outcome("op-002", 0.0, &mut rng);
        assert!(!outcome.success);

        // Mid-range success rate (deterministic with fixed seed)
        let _outcome = FactionService::generate_outcome("op-003", 0.5, &mut rng);
        // Either outcome is valid for 0.5 probability
    }

    #[test]
    fn test_generate_outcome_with_metrics() {
        let mut rng = StdRng::seed_from_u64(42);

        let success_metrics =
            HashMap::from([("territory_gained".into(), 1.0), ("casualties".into(), 5.0)]);

        let failure_metrics =
            HashMap::from([("casualties".into(), 15.0), ("morale_loss".into(), 10.0)]);

        // 100% success rate
        let outcome = FactionService::generate_outcome_with_metrics(
            "op-001",
            1.0,
            &mut rng,
            success_metrics.clone(),
            failure_metrics.clone(),
        );
        assert!(outcome.success);
        assert_eq!(outcome.metrics.get("territory_gained"), Some(&1.0));
        assert_eq!(outcome.metrics.get("casualties"), Some(&5.0));

        // 0% success rate
        let outcome = FactionService::generate_outcome_with_metrics(
            "op-002",
            0.0,
            &mut rng,
            success_metrics,
            failure_metrics,
        );
        assert!(!outcome.success);
        assert_eq!(outcome.metrics.get("casualties"), Some(&15.0));
        assert_eq!(outcome.metrics.get("morale_loss"), Some(&10.0));
    }

    #[test]
    fn test_calculate_relationship_change() {
        // Successful operation
        let change = FactionService::calculate_relationship_change(10.0, true, 1.5, 0.5);
        assert_eq!(change, 15.0); // 10 * 1.5

        // Failed operation
        let change = FactionService::calculate_relationship_change(10.0, false, 1.5, 0.5);
        assert_eq!(change, 5.0); // 10 * 0.5

        // Negative base change (penalty)
        let change = FactionService::calculate_relationship_change(-10.0, false, 1.5, 0.5);
        assert_eq!(change, -5.0); // -10 * 0.5
    }

    #[test]
    fn test_estimate_operation_effectiveness() {
        // 70% success, +100 on success, -30 on failure
        let effectiveness = FactionService::estimate_operation_effectiveness(0.7, 100.0, -30.0);
        assert!((effectiveness - 61.0).abs() < 0.001); // (0.7 * 100) + (0.3 * -30) = 61

        // 100% success
        let effectiveness = FactionService::estimate_operation_effectiveness(1.0, 100.0, -30.0);
        assert_eq!(effectiveness, 100.0);

        // 0% success
        let effectiveness = FactionService::estimate_operation_effectiveness(0.0, 100.0, -30.0);
        assert_eq!(effectiveness, -30.0);

        // 50/50
        let effectiveness = FactionService::estimate_operation_effectiveness(0.5, 100.0, -100.0);
        assert_eq!(effectiveness, 0.0); // (0.5 * 100) + (0.5 * -100) = 0
    }
}
