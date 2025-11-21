//! Policy definitions (ReadOnly asset)

use super::types::*;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection of policy definitions (ReadOnly)
///
/// This is an asset loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policies {
    policies: HashMap<PolicyId, Policy>,
}

impl Resource for Policies {}

impl Policies {
    /// Create a new empty policies collection
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Add a policy definition
    pub fn add(&mut self, policy: Policy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    /// Get a policy by id
    pub fn get(&self, id: &PolicyId) -> Option<&Policy> {
        self.policies.get(id)
    }

    /// Check if a policy exists
    pub fn contains(&self, id: &PolicyId) -> bool {
        self.policies.contains_key(id)
    }

    /// List all available policies
    pub fn iter(&self) -> impl Iterator<Item = &Policy> {
        self.policies.values()
    }

    /// Get all policy IDs (sorted alphabetically)
    pub fn policy_ids(&self) -> Vec<PolicyId> {
        let mut ids: Vec<_> = self.policies.keys().cloned().collect();
        ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        ids
    }

    /// Get the number of policies
    pub fn len(&self) -> usize {
        self.policies.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }
}

impl Default for Policies {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy(id: &str, name: &str) -> Policy {
        Policy::new(id, name, "Test policy").add_effect("income_multiplier", 1.2)
    }

    #[test]
    fn test_policies_new() {
        let policies = Policies::new();
        assert!(policies.is_empty());
        assert_eq!(policies.len(), 0);
    }

    #[test]
    fn test_add_and_get() {
        let mut policies = Policies::new();
        let policy = create_test_policy("test", "Test Policy");
        policies.add(policy);

        assert_eq!(policies.len(), 1);
        assert!(!policies.is_empty());
        assert!(policies.get(&PolicyId::new("test")).is_some());
        assert_eq!(
            policies.get(&PolicyId::new("test")).unwrap().name,
            "Test Policy"
        );
    }

    #[test]
    fn test_contains() {
        let mut policies = Policies::new();
        policies.add(create_test_policy("test", "Test"));

        assert!(policies.contains(&PolicyId::new("test")));
        assert!(!policies.contains(&PolicyId::new("other")));
    }

    #[test]
    fn test_iter() {
        let mut policies = Policies::new();
        policies.add(create_test_policy("p1", "P1"));
        policies.add(create_test_policy("p2", "P2"));

        let count = policies.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_policy_ids_sorted() {
        let mut policies = Policies::new();
        policies.add(create_test_policy("c_policy", "C"));
        policies.add(create_test_policy("a_policy", "A"));
        policies.add(create_test_policy("b_policy", "B"));

        let ids = policies.policy_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0].as_str(), "a_policy");
        assert_eq!(ids[1].as_str(), "b_policy");
        assert_eq!(ids[2].as_str(), "c_policy");
    }
}
