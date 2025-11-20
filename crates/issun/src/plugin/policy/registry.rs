//! Policy registry and configuration

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for PolicyRegistry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Allow multiple policies to be active simultaneously
    pub allow_multiple_active: bool,

    /// Currently active policies (when allow_multiple_active = true)
    pub active_policy_ids: Vec<PolicyId>,

    /// Maximum number of active policies (when allow_multiple_active = true)
    pub max_active_policies: Option<usize>,

    /// Enable policy cycling (activate next policy in registry)
    pub enable_cycling: bool,

    /// Effect-specific aggregation strategies
    ///
    /// Maps effect names to their aggregation strategies.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut strategies = HashMap::new();
    /// strategies.insert("income_multiplier".into(), AggregationStrategy::Multiply);
    /// strategies.insert("attack_bonus".into(), AggregationStrategy::Add);
    /// strategies.insert("build_cost".into(), AggregationStrategy::Min);
    /// ```
    pub aggregation_strategies: HashMap<String, AggregationStrategy>,

    /// Default aggregation strategy (when effect not in aggregation_strategies map)
    pub default_aggregation: AggregationStrategy,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allow_multiple_active: false,
            active_policy_ids: Vec::new(),
            max_active_policies: None,
            enable_cycling: true,
            aggregation_strategies: HashMap::new(),
            default_aggregation: AggregationStrategy::Multiply,
        }
    }
}

/// Registry of all policies in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRegistry {
    /// All available policies
    policies: HashMap<PolicyId, Policy>,

    /// Currently active policy (None if no policy is active)
    active_policy_id: Option<PolicyId>,

    /// Configuration
    config: PolicyConfig,
}

impl PolicyRegistry {
    /// Create a new policy registry
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            active_policy_id: None,
            config: PolicyConfig::default(),
        }
    }

    /// Create a policy registry with custom configuration
    pub fn with_config(config: PolicyConfig) -> Self {
        Self {
            policies: HashMap::new(),
            active_policy_id: None,
            config,
        }
    }

    /// Add a policy to the registry
    pub fn add(&mut self, policy: Policy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    /// Get a policy by id
    pub fn get(&self, id: &PolicyId) -> Option<&Policy> {
        self.policies.get(id)
    }

    /// Get mutable policy by id
    pub fn get_mut(&mut self, id: &PolicyId) -> Option<&mut Policy> {
        self.policies.get_mut(id)
    }

    /// Get the currently active policy (single-active mode)
    pub fn active_policy(&self) -> Option<&Policy> {
        self.active_policy_id
            .as_ref()
            .and_then(|id| self.policies.get(id))
    }

    /// Get all active policies (multi-active mode)
    pub fn active_policies(&self) -> Vec<&Policy> {
        self.config
            .active_policy_ids
            .iter()
            .filter_map(|id| self.policies.get(id))
            .collect()
    }

    /// Activate a policy (single-active mode)
    ///
    /// Returns the previously active policy (if any).
    pub fn activate(&mut self, id: &PolicyId) -> Result<Option<PolicyId>, PolicyError> {
        if !self.policies.contains_key(id) {
            return Err(PolicyError::NotFound);
        }

        let previous = self.active_policy_id.take();
        self.active_policy_id = Some(id.clone());
        Ok(previous)
    }

    /// Activate a policy (multi-active mode)
    ///
    /// Returns error if policy not found or max active policies reached.
    pub fn activate_multi(&mut self, id: &PolicyId) -> Result<(), PolicyError> {
        if !self.policies.contains_key(id) {
            return Err(PolicyError::NotFound);
        }

        if self.config.active_policy_ids.contains(id) {
            return Err(PolicyError::AlreadyActive);
        }

        if let Some(max) = self.config.max_active_policies {
            if self.config.active_policy_ids.len() >= max {
                return Err(PolicyError::MaxActivePoliciesReached);
            }
        }

        self.config.active_policy_ids.push(id.clone());
        Ok(())
    }

    /// Deactivate the current policy (single-active mode)
    pub fn deactivate(&mut self) -> Option<PolicyId> {
        self.active_policy_id.take()
    }

    /// Deactivate a specific policy (multi-active mode)
    pub fn deactivate_multi(&mut self, id: &PolicyId) -> Result<(), PolicyError> {
        let index = self
            .config
            .active_policy_ids
            .iter()
            .position(|p| p == id)
            .ok_or(PolicyError::NoActivePolicy)?;

        self.config.active_policy_ids.remove(index);
        Ok(())
    }

    /// Cycle to the next policy (single-active mode)
    ///
    /// Activates the next policy in the registry (alphabetically by ID).
    /// If no policy is active, activates the first one.
    /// If the last policy is active, wraps around to the first.
    pub fn cycle(&mut self) -> Result<Option<PolicyId>, PolicyError> {
        if !self.config.enable_cycling {
            return Err(PolicyError::CyclingDisabled);
        }

        let mut policy_ids: Vec<_> = self.policies.keys().cloned().collect();
        policy_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        if policy_ids.is_empty() {
            return Ok(None);
        }

        let next_id = if let Some(current_id) = &self.active_policy_id {
            let current_index = policy_ids
                .iter()
                .position(|id| id == current_id)
                .unwrap_or(0);
            let next_index = (current_index + 1) % policy_ids.len();
            policy_ids[next_index].clone()
        } else {
            policy_ids[0].clone()
        };

        let previous = self.active_policy_id.take();
        self.active_policy_id = Some(next_id);
        Ok(previous)
    }

    /// List all available policies
    pub fn iter(&self) -> impl Iterator<Item = &Policy> {
        self.policies.values()
    }

    /// Get aggregated effects from all active policies
    ///
    /// Effects are aggregated according to their configured AggregationStrategy.
    ///
    /// # Examples
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
    pub fn aggregate_effects(&self) -> HashMap<String, f32> {
        let active_policies = if self.config.allow_multiple_active {
            self.active_policies()
        } else {
            self.active_policy().into_iter().collect()
        };

        let mut aggregated = HashMap::new();
        for policy in active_policies {
            for (key, value) in &policy.effects {
                // Determine aggregation strategy for this effect
                let strategy = self
                    .config
                    .aggregation_strategies
                    .get(key)
                    .copied()
                    .unwrap_or(self.config.default_aggregation);

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

    /// Get a specific effect value (with appropriate fallback based on aggregation strategy)
    ///
    /// # Fallback values
    ///
    /// - **Multiply**: 1.0 (neutral multiplier)
    /// - **Add**: 0.0 (no bonus)
    /// - **Max**: f32::MIN (no cap)
    /// - **Min**: f32::MAX (no reduction)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiply strategy (default)
    /// let income_mult = registry.get_effect("income_multiplier"); // Returns 1.2 or 1.0 (default)
    ///
    /// // Add strategy
    /// let attack_bonus = registry.get_effect("attack_bonus"); // Returns 15.0 or 0.0 (default)
    /// ```
    pub fn get_effect(&self, effect_name: &str) -> f32 {
        if let Some(value) = self.aggregate_effects().get(effect_name) {
            return *value;
        }

        // Return appropriate fallback based on aggregation strategy
        let strategy = self
            .config
            .aggregation_strategies
            .get(effect_name)
            .copied()
            .unwrap_or(self.config.default_aggregation);

        strategy.initial_value()
    }

    /// Get the configuration
    pub fn config(&self) -> &PolicyConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut PolicyConfig {
        &mut self.config
    }

    /// Get the number of policies in the registry
    pub fn len(&self) -> usize {
        self.policies.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }
}

impl Default for PolicyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when working with policies
#[derive(Debug, Clone, thiserror::Error)]
pub enum PolicyError {
    #[error("Policy not found")]
    NotFound,

    #[error("Policy already active")]
    AlreadyActive,

    #[error("No active policy")]
    NoActivePolicy,

    #[error("Maximum number of active policies reached")]
    MaxActivePoliciesReached,

    #[error("Policy cycling is disabled")]
    CyclingDisabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy(id: &str, name: &str) -> Policy {
        Policy::new(id, name, "Test policy")
            .add_effect("income_multiplier", 1.2)
            .add_effect("attack_bonus", 10.0)
    }

    #[test]
    fn test_registry_creation() {
        let registry = PolicyRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.active_policy().is_none());
    }

    #[test]
    fn test_add_policy() {
        let mut registry = PolicyRegistry::new();
        let policy = create_test_policy("test", "Test Policy");
        registry.add(policy);

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.get(&PolicyId::new("test")).is_some());
    }

    #[test]
    fn test_activate_policy() {
        let mut registry = PolicyRegistry::new();
        registry.add(create_test_policy("policy1", "Policy 1"));
        registry.add(create_test_policy("policy2", "Policy 2"));

        let result = registry.activate(&PolicyId::new("policy1"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // No previous policy

        assert_eq!(
            registry.active_policy().unwrap().id.as_str(),
            "policy1"
        );
    }

    #[test]
    fn test_activate_policy_switches_previous() {
        let mut registry = PolicyRegistry::new();
        registry.add(create_test_policy("policy1", "Policy 1"));
        registry.add(create_test_policy("policy2", "Policy 2"));

        registry.activate(&PolicyId::new("policy1")).unwrap();
        let result = registry.activate(&PolicyId::new("policy2"));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(PolicyId::new("policy1"))); // Previous policy

        assert_eq!(
            registry.active_policy().unwrap().id.as_str(),
            "policy2"
        );
    }

    #[test]
    fn test_activate_nonexistent_policy() {
        let mut registry = PolicyRegistry::new();
        let result = registry.activate(&PolicyId::new("nonexistent"));
        assert!(matches!(result, Err(PolicyError::NotFound)));
    }

    #[test]
    fn test_deactivate_policy() {
        let mut registry = PolicyRegistry::new();
        registry.add(create_test_policy("policy1", "Policy 1"));
        registry.activate(&PolicyId::new("policy1")).unwrap();

        let deactivated = registry.deactivate();
        assert_eq!(deactivated, Some(PolicyId::new("policy1")));
        assert!(registry.active_policy().is_none());
    }

    #[test]
    fn test_aggregate_effects_multiply() {
        let mut config = PolicyConfig::default();
        config.allow_multiple_active = true;
        config
            .aggregation_strategies
            .insert("income_multiplier".into(), AggregationStrategy::Multiply);

        let mut registry = PolicyRegistry::with_config(config);

        let policy1 = Policy::new("p1", "P1", "P1")
            .add_effect("income_multiplier", 1.2);
        let policy2 = Policy::new("p2", "P2", "P2")
            .add_effect("income_multiplier", 1.1);

        registry.add(policy1);
        registry.add(policy2);

        registry.activate_multi(&PolicyId::new("p1")).unwrap();
        registry.activate_multi(&PolicyId::new("p2")).unwrap();

        let effects = registry.aggregate_effects();
        assert_eq!(effects.get("income_multiplier"), Some(&1.32)); // 1.2 * 1.1
    }

    #[test]
    fn test_aggregate_effects_add() {
        let mut config = PolicyConfig::default();
        config.allow_multiple_active = true;
        config
            .aggregation_strategies
            .insert("attack_bonus".into(), AggregationStrategy::Add);

        let mut registry = PolicyRegistry::with_config(config);

        let policy1 = Policy::new("p1", "P1", "P1").add_effect("attack_bonus", 10.0);
        let policy2 = Policy::new("p2", "P2", "P2").add_effect("attack_bonus", 5.0);

        registry.add(policy1);
        registry.add(policy2);

        registry.activate_multi(&PolicyId::new("p1")).unwrap();
        registry.activate_multi(&PolicyId::new("p2")).unwrap();

        let effects = registry.aggregate_effects();
        assert_eq!(effects.get("attack_bonus"), Some(&15.0)); // 10 + 5
    }

    #[test]
    fn test_aggregate_effects_min() {
        let mut config = PolicyConfig::default();
        config.allow_multiple_active = true;
        config
            .aggregation_strategies
            .insert("build_cost".into(), AggregationStrategy::Min);

        let mut registry = PolicyRegistry::with_config(config);

        let policy1 = Policy::new("p1", "P1", "P1").add_effect("build_cost", 0.9);
        let policy2 = Policy::new("p2", "P2", "P2").add_effect("build_cost", 0.8);

        registry.add(policy1);
        registry.add(policy2);

        registry.activate_multi(&PolicyId::new("p1")).unwrap();
        registry.activate_multi(&PolicyId::new("p2")).unwrap();

        let effects = registry.aggregate_effects();
        assert_eq!(effects.get("build_cost"), Some(&0.8)); // min(0.9, 0.8)
    }

    #[test]
    fn test_aggregate_effects_max() {
        let mut config = PolicyConfig::default();
        config.allow_multiple_active = true;
        config
            .aggregation_strategies
            .insert("max_speed".into(), AggregationStrategy::Max);

        let mut registry = PolicyRegistry::with_config(config);

        let policy1 = Policy::new("p1", "P1", "P1").add_effect("max_speed", 1.2);
        let policy2 = Policy::new("p2", "P2", "P2").add_effect("max_speed", 1.1);

        registry.add(policy1);
        registry.add(policy2);

        registry.activate_multi(&PolicyId::new("p1")).unwrap();
        registry.activate_multi(&PolicyId::new("p2")).unwrap();

        let effects = registry.aggregate_effects();
        assert_eq!(effects.get("max_speed"), Some(&1.2)); // max(1.2, 1.1)
    }

    #[test]
    fn test_get_effect_with_fallback() {
        let registry = PolicyRegistry::new();

        // No active policy, should return default (1.0 for Multiply)
        assert_eq!(registry.get_effect("income_multiplier"), 1.0);

        // Add strategy for attack_bonus (Add)
        let mut registry = registry;
        registry
            .config
            .aggregation_strategies
            .insert("attack_bonus".into(), AggregationStrategy::Add);

        // Should return 0.0 for Add strategy
        assert_eq!(registry.get_effect("attack_bonus"), 0.0);
    }

    #[test]
    fn test_multi_active_max_limit() {
        let mut config = PolicyConfig::default();
        config.allow_multiple_active = true;
        config.max_active_policies = Some(2);

        let mut registry = PolicyRegistry::with_config(config);

        registry.add(create_test_policy("p1", "P1"));
        registry.add(create_test_policy("p2", "P2"));
        registry.add(create_test_policy("p3", "P3"));

        registry.activate_multi(&PolicyId::new("p1")).unwrap();
        registry.activate_multi(&PolicyId::new("p2")).unwrap();

        let result = registry.activate_multi(&PolicyId::new("p3"));
        assert!(matches!(
            result,
            Err(PolicyError::MaxActivePoliciesReached)
        ));
    }

    #[test]
    fn test_cycle_policies() {
        let mut registry = PolicyRegistry::new();
        registry.add(create_test_policy("a_policy", "A"));
        registry.add(create_test_policy("b_policy", "B"));
        registry.add(create_test_policy("c_policy", "C"));

        // First cycle: no active policy -> activates first (alphabetically)
        registry.cycle().unwrap();
        assert_eq!(registry.active_policy().unwrap().id.as_str(), "a_policy");

        // Second cycle: a -> b
        registry.cycle().unwrap();
        assert_eq!(registry.active_policy().unwrap().id.as_str(), "b_policy");

        // Third cycle: b -> c
        registry.cycle().unwrap();
        assert_eq!(registry.active_policy().unwrap().id.as_str(), "c_policy");

        // Fourth cycle: c -> a (wrap around)
        registry.cycle().unwrap();
        assert_eq!(registry.active_policy().unwrap().id.as_str(), "a_policy");
    }

    #[test]
    fn test_cycle_disabled() {
        let mut config = PolicyConfig::default();
        config.enable_cycling = false;

        let mut registry = PolicyRegistry::with_config(config);
        registry.add(create_test_policy("policy1", "Policy 1"));

        let result = registry.cycle();
        assert!(matches!(result, Err(PolicyError::CyclingDisabled)));
    }
}
