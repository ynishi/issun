//! Policy service for domain logic
//!
//! Provides pure functions for policy effect calculations and aggregations.
//! All functions are stateless and can be used independently.

use super::policies::Policies;
use super::state::PolicyState;
use super::types::{AggregationStrategy, Policy};
use crate::context::ResourceContext;
use std::collections::HashMap;

/// Policy service providing pure policy calculation logic
///
/// This service handles stateless calculations for policy operations.
/// It follows Domain-Driven Design principles - policy logic as a service.
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
/// use issun::plugin::policy::{PolicyService, Policy, AggregationStrategy};
/// use std::collections::HashMap;
///
/// let policy1 = Policy::new("p1", "P1", "Policy 1")
///     .add_effect("income_multiplier", 1.2);
/// let policy2 = Policy::new("p2", "P2", "Policy 2")
///     .add_effect("income_multiplier", 1.1);
///
/// let policies = vec![&policy1, &policy2];
/// let mut strategies = HashMap::new();
/// strategies.insert("income_multiplier".into(), AggregationStrategy::Multiply);
///
/// let effects = PolicyService::aggregate_effects(
///     &policies,
///     &strategies,
///     AggregationStrategy::Multiply,
/// );
///
/// assert_eq!(effects.get("income_multiplier"), Some(&1.32)); // 1.2 * 1.1
/// ```
#[derive(Debug, Clone, Default)]
pub struct PolicyService;

impl PolicyService {
    /// Create a new policy service
    pub fn new() -> Self {
        Self
    }

    /// Aggregate effects from multiple policies
    ///
    /// This is the core aggregation logic. It combines effects from multiple
    /// policies according to their aggregation strategies.
    ///
    /// # Arguments
    ///
    /// * `policies` - Active policies to aggregate effects from
    /// * `strategies` - Effect-specific aggregation strategies
    /// * `default_strategy` - Strategy to use when effect not in strategies map
    ///
    /// # Returns
    ///
    /// HashMap of aggregated effects with their final values
    ///
    /// # Aggregation Examples
    ///
    /// **Multiply (default)**:
    /// ```ignore
    /// Policy A: { "income_multiplier": 1.2 }
    /// Policy B: { "income_multiplier": 1.1 }
    /// Result: { "income_multiplier": 1.32 }  // 1.2 * 1.1
    /// ```
    ///
    /// **Add**:
    /// ```ignore
    /// Policy A: { "attack_bonus": 10.0 }
    /// Policy B: { "attack_bonus": 5.0 }
    /// Result: { "attack_bonus": 15.0 }  // 10 + 5
    /// ```
    ///
    /// **Min**:
    /// ```ignore
    /// Policy A: { "build_cost": 0.9 }
    /// Policy B: { "build_cost": 0.8 }
    /// Result: { "build_cost": 0.8 }  // min(0.9, 0.8)
    /// ```
    ///
    /// **Max**:
    /// ```ignore
    /// Policy A: { "max_speed": 1.2 }
    /// Policy B: { "max_speed": 1.1 }
    /// Result: { "max_speed": 1.2 }  // max(1.2, 1.1)
    /// ```
    pub fn aggregate_effects(
        policies: &[&Policy],
        strategies: &HashMap<String, AggregationStrategy>,
        default_strategy: AggregationStrategy,
    ) -> HashMap<String, f32> {
        let mut aggregated = HashMap::new();

        for policy in policies {
            for (key, value) in &policy.effects {
                // Determine aggregation strategy for this effect
                let strategy = strategies.get(key).copied().unwrap_or(default_strategy);

                // Get current aggregated value (with appropriate initial value)
                let current = aggregated
                    .get(key)
                    .copied()
                    .unwrap_or_else(|| strategy.initial_value());

                // Apply aggregation strategy
                let new_value = strategy.aggregate(current, *value);

                aggregated.insert(key.clone(), new_value);
            }
        }

        aggregated
    }

    /// Get a specific effect value from aggregated effects
    ///
    /// Returns the effect value if it exists, otherwise returns the appropriate
    /// fallback value based on the aggregation strategy.
    ///
    /// # Fallback values
    ///
    /// - **Multiply**: 1.0 (neutral multiplier)
    /// - **Add**: 0.0 (no bonus)
    /// - **Max**: f32::MIN (no cap)
    /// - **Min**: f32::MAX (no reduction)
    ///
    /// # Arguments
    ///
    /// * `effects` - Aggregated effects map
    /// * `effect_name` - Name of the effect to retrieve
    /// * `strategies` - Effect-specific aggregation strategies
    /// * `default_strategy` - Strategy to use when effect not in strategies map
    ///
    /// # Returns
    ///
    /// Effect value or appropriate fallback
    ///
    /// # Example
    ///
    /// ```ignore
    /// let effects = HashMap::new();
    /// let strategies = HashMap::new();
    ///
    /// // Multiply strategy (default) returns 1.0 when effect not found
    /// let income = PolicyService::get_effect(
    ///     &effects,
    ///     "income_multiplier",
    ///     &strategies,
    ///     AggregationStrategy::Multiply,
    /// );
    /// assert_eq!(income, 1.0);
    ///
    /// // Add strategy returns 0.0 when effect not found
    /// let mut add_strategies = HashMap::new();
    /// add_strategies.insert("attack_bonus".into(), AggregationStrategy::Add);
    /// let attack = PolicyService::get_effect(
    ///     &effects,
    ///     "attack_bonus",
    ///     &add_strategies,
    ///     AggregationStrategy::Multiply,
    /// );
    /// assert_eq!(attack, 0.0);
    /// ```
    pub fn get_effect(
        effects: &HashMap<String, f32>,
        effect_name: &str,
        strategies: &HashMap<String, AggregationStrategy>,
        default_strategy: AggregationStrategy,
    ) -> f32 {
        if let Some(value) = effects.get(effect_name) {
            return *value;
        }

        // Return appropriate fallback based on aggregation strategy
        let strategy = strategies
            .get(effect_name)
            .copied()
            .unwrap_or(default_strategy);

        strategy.initial_value()
    }

    /// Apply a single effect to a base value
    ///
    /// This is useful for applying individual policy effects to game values.
    ///
    /// # Arguments
    ///
    /// * `base_value` - The original value before policy effect
    /// * `effect_value` - The policy effect value
    /// * `strategy` - How to apply the effect
    ///
    /// # Returns
    ///
    /// New value after applying the effect
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiply: income * policy effect
    /// let income = 100.0;
    /// let policy_effect = 1.2;
    /// let new_income = PolicyService::apply_effect(
    ///     income,
    ///     policy_effect,
    ///     AggregationStrategy::Multiply,
    /// );
    /// assert_eq!(new_income, 120.0); // 100 * 1.2
    ///
    /// // Add: attack + policy bonus
    /// let attack = 50.0;
    /// let policy_bonus = 10.0;
    /// let new_attack = PolicyService::apply_effect(
    ///     attack,
    ///     policy_bonus,
    ///     AggregationStrategy::Add,
    /// );
    /// assert_eq!(new_attack, 60.0); // 50 + 10
    /// ```
    pub fn apply_effect(
        base_value: f32,
        effect_value: f32,
        strategy: AggregationStrategy,
    ) -> f32 {
        strategy.aggregate(base_value, effect_value)
    }

    /// Calculate the combined effect of multiple values using a strategy
    ///
    /// This is a generalized aggregation function for combining multiple values.
    ///
    /// # Arguments
    ///
    /// * `values` - Values to combine
    /// * `strategy` - How to combine them
    ///
    /// # Returns
    ///
    /// Combined value
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiply: 1.2 * 1.1 * 1.15
    /// let multipliers = vec![1.2, 1.1, 1.15];
    /// let combined = PolicyService::combine_values(
    ///     &multipliers,
    ///     AggregationStrategy::Multiply,
    /// );
    /// assert_eq!(combined, 1.518); // 1.2 * 1.1 * 1.15
    ///
    /// // Add: 10 + 5 + 3
    /// let bonuses = vec![10.0, 5.0, 3.0];
    /// let total = PolicyService::combine_values(
    ///     &bonuses,
    ///     AggregationStrategy::Add,
    /// );
    /// assert_eq!(total, 18.0); // 10 + 5 + 3
    /// ```
    pub fn combine_values(values: &[f32], strategy: AggregationStrategy) -> f32 {
        values
            .iter()
            .fold(strategy.initial_value(), |acc, value| {
                strategy.aggregate(acc, *value)
            })
    }

    // ========================================
    // ResourceContext Helpers (for Hooks)
    // ========================================

    /// Get active policy effect value from ResourceContext
    ///
    /// This is a convenience helper for Hooks to easily access active policy effects
    /// without manually combining PolicyState and Policies.
    ///
    /// # Arguments
    ///
    /// * `effect_name` - Name of the effect to retrieve
    /// * `resources` - Resource context containing PolicyState and Policies
    ///
    /// # Returns
    ///
    /// Effect value or 1.0 (neutral multiplier) if not found
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In a Hook
    /// async fn calculate_income(&self, resources: &ResourceContext) -> Currency {
    ///     let multiplier = PolicyService::get_active_effect("income_multiplier", resources).await;
    ///     Currency::new((base_income as f32 * multiplier) as i64)
    /// }
    /// ```
    pub async fn get_active_effect(effect_name: &str, resources: &ResourceContext) -> f32 {
        let state = match resources.get::<PolicyState>().await {
            Some(s) => s,
            None => return 1.0, // Default multiplier
        };

        let policies = match resources.get::<Policies>().await {
            Some(p) => p,
            None => return 1.0,
        };

        // Get active policy ID (single-active mode)
        if let Some(active_id) = state.active_policy_id() {
            if let Some(policy) = policies.get(active_id) {
                return policy.effects.get(effect_name).copied().unwrap_or(1.0);
            }
        }

        1.0 // Default multiplier
    }

    /// Get all active policy effects from ResourceContext
    ///
    /// This is a convenience helper for Hooks to easily access all active policy effects.
    ///
    /// # Arguments
    ///
    /// * `resources` - Resource context containing PolicyState and Policies
    ///
    /// # Returns
    ///
    /// HashMap of all aggregated effects from active policies
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In a Hook
    /// let effects = PolicyService::get_active_effects(resources).await;
    /// let income_mult = effects.get("income_multiplier").copied().unwrap_or(1.0);
    /// let attack_bonus = effects.get("attack_bonus").copied().unwrap_or(0.0);
    /// ```
    pub async fn get_active_effects(resources: &ResourceContext) -> HashMap<String, f32> {
        let state = match resources.get::<PolicyState>().await {
            Some(s) => s,
            None => return HashMap::new(),
        };

        let policies_resource = match resources.get::<Policies>().await {
            Some(p) => p,
            None => return HashMap::new(),
        };

        // Collect active policies
        let active_policies: Vec<&Policy> = if let Some(active_id) = state.active_policy_id() {
            if let Some(policy) = policies_resource.get(active_id) {
                vec![policy]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        if active_policies.is_empty() {
            return HashMap::new();
        }

        // Aggregate effects (default to Multiply strategy)
        Self::aggregate_effects(
            &active_policies,
            &HashMap::new(),
            AggregationStrategy::Multiply,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy(id: &str, effects: Vec<(&str, f32)>) -> Policy {
        let mut policy = Policy::new(id, id, "Test policy");
        for (key, value) in effects {
            policy = policy.add_effect(key, value);
        }
        policy
    }

    #[test]
    fn test_aggregate_effects_multiply() {
        let policy1 = create_test_policy("p1", vec![("income_multiplier", 1.2)]);
        let policy2 = create_test_policy("p2", vec![("income_multiplier", 1.1)]);

        let policies = vec![&policy1, &policy2];
        let mut strategies = HashMap::new();
        strategies.insert("income_multiplier".into(), AggregationStrategy::Multiply);

        let effects =
            PolicyService::aggregate_effects(&policies, &strategies, AggregationStrategy::Multiply);

        assert_eq!(effects.get("income_multiplier"), Some(&1.32)); // 1.2 * 1.1
    }

    #[test]
    fn test_aggregate_effects_add() {
        let policy1 = create_test_policy("p1", vec![("attack_bonus", 10.0)]);
        let policy2 = create_test_policy("p2", vec![("attack_bonus", 5.0)]);

        let policies = vec![&policy1, &policy2];
        let mut strategies = HashMap::new();
        strategies.insert("attack_bonus".into(), AggregationStrategy::Add);

        let effects =
            PolicyService::aggregate_effects(&policies, &strategies, AggregationStrategy::Multiply);

        assert_eq!(effects.get("attack_bonus"), Some(&15.0)); // 10 + 5
    }

    #[test]
    fn test_aggregate_effects_min() {
        let policy1 = create_test_policy("p1", vec![("build_cost", 0.9)]);
        let policy2 = create_test_policy("p2", vec![("build_cost", 0.8)]);

        let policies = vec![&policy1, &policy2];
        let mut strategies = HashMap::new();
        strategies.insert("build_cost".into(), AggregationStrategy::Min);

        let effects =
            PolicyService::aggregate_effects(&policies, &strategies, AggregationStrategy::Multiply);

        assert_eq!(effects.get("build_cost"), Some(&0.8)); // min(0.9, 0.8)
    }

    #[test]
    fn test_aggregate_effects_max() {
        let policy1 = create_test_policy("p1", vec![("max_speed", 1.2)]);
        let policy2 = create_test_policy("p2", vec![("max_speed", 1.1)]);

        let policies = vec![&policy1, &policy2];
        let mut strategies = HashMap::new();
        strategies.insert("max_speed".into(), AggregationStrategy::Max);

        let effects =
            PolicyService::aggregate_effects(&policies, &strategies, AggregationStrategy::Multiply);

        assert_eq!(effects.get("max_speed"), Some(&1.2)); // max(1.2, 1.1)
    }

    #[test]
    fn test_aggregate_effects_mixed_strategies() {
        let policy1 = create_test_policy(
            "p1",
            vec![
                ("income_multiplier", 1.2),
                ("attack_bonus", 10.0),
                ("build_cost", 0.9),
            ],
        );
        let policy2 = create_test_policy(
            "p2",
            vec![
                ("income_multiplier", 1.1),
                ("attack_bonus", 5.0),
                ("build_cost", 0.8),
            ],
        );

        let policies = vec![&policy1, &policy2];
        let mut strategies = HashMap::new();
        strategies.insert("income_multiplier".into(), AggregationStrategy::Multiply);
        strategies.insert("attack_bonus".into(), AggregationStrategy::Add);
        strategies.insert("build_cost".into(), AggregationStrategy::Min);

        let effects =
            PolicyService::aggregate_effects(&policies, &strategies, AggregationStrategy::Multiply);

        assert_eq!(effects.get("income_multiplier"), Some(&1.32)); // 1.2 * 1.1
        assert_eq!(effects.get("attack_bonus"), Some(&15.0)); // 10 + 5
        assert_eq!(effects.get("build_cost"), Some(&0.8)); // min(0.9, 0.8)
    }

    #[test]
    fn test_aggregate_effects_empty_policies() {
        let policies: Vec<&Policy> = vec![];
        let strategies = HashMap::new();

        let effects =
            PolicyService::aggregate_effects(&policies, &strategies, AggregationStrategy::Multiply);

        assert!(effects.is_empty());
    }

    #[test]
    fn test_get_effect_existing() {
        let mut effects = HashMap::new();
        effects.insert("income_multiplier".into(), 1.5);

        let strategies = HashMap::new();

        let value = PolicyService::get_effect(
            &effects,
            "income_multiplier",
            &strategies,
            AggregationStrategy::Multiply,
        );

        assert_eq!(value, 1.5);
    }

    #[test]
    fn test_get_effect_fallback_multiply() {
        let effects = HashMap::new();
        let strategies = HashMap::new();

        let value = PolicyService::get_effect(
            &effects,
            "income_multiplier",
            &strategies,
            AggregationStrategy::Multiply,
        );

        assert_eq!(value, 1.0); // Multiply fallback
    }

    #[test]
    fn test_get_effect_fallback_add() {
        let effects = HashMap::new();
        let mut strategies = HashMap::new();
        strategies.insert("attack_bonus".into(), AggregationStrategy::Add);

        let value = PolicyService::get_effect(
            &effects,
            "attack_bonus",
            &strategies,
            AggregationStrategy::Multiply,
        );

        assert_eq!(value, 0.0); // Add fallback
    }

    #[test]
    fn test_apply_effect_multiply() {
        let result = PolicyService::apply_effect(100.0, 1.2, AggregationStrategy::Multiply);
        assert!((result - 120.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_effect_add() {
        let result = PolicyService::apply_effect(50.0, 10.0, AggregationStrategy::Add);
        assert_eq!(result, 60.0);
    }

    #[test]
    fn test_apply_effect_min() {
        let result = PolicyService::apply_effect(0.9, 0.8, AggregationStrategy::Min);
        assert_eq!(result, 0.8);
    }

    #[test]
    fn test_apply_effect_max() {
        let result = PolicyService::apply_effect(1.2, 1.5, AggregationStrategy::Max);
        assert_eq!(result, 1.5);
    }

    #[test]
    fn test_combine_values_multiply() {
        let values = vec![1.2, 1.1, 1.15];
        let result = PolicyService::combine_values(&values, AggregationStrategy::Multiply);
        assert!((result - 1.518).abs() < 0.001); // 1.2 * 1.1 * 1.15 ≈ 1.518
    }

    #[test]
    fn test_combine_values_add() {
        let values = vec![10.0, 5.0, 3.0];
        let result = PolicyService::combine_values(&values, AggregationStrategy::Add);
        assert_eq!(result, 18.0);
    }

    #[test]
    fn test_combine_values_min() {
        let values = vec![0.9, 0.8, 0.85];
        let result = PolicyService::combine_values(&values, AggregationStrategy::Min);
        assert_eq!(result, 0.8);
    }

    #[test]
    fn test_combine_values_max() {
        let values = vec![1.2, 1.5, 1.3];
        let result = PolicyService::combine_values(&values, AggregationStrategy::Max);
        assert_eq!(result, 1.5);
    }

    #[test]
    fn test_combine_values_empty() {
        let values: Vec<f32> = vec![];

        // Empty multiply returns 1.0 (identity)
        let result = PolicyService::combine_values(&values, AggregationStrategy::Multiply);
        assert_eq!(result, 1.0);

        // Empty add returns 0.0 (identity)
        let result = PolicyService::combine_values(&values, AggregationStrategy::Add);
        assert_eq!(result, 0.0);
    }
}
