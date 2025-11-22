//! State management for ChainOfCommandPlugin
//!
//! Provides OrganizationHierarchy and HierarchyState for managing
//! organizational command structures across multiple factions.

use super::types::{FactionId, Member, MemberId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hierarchy structure for a single organization
///
/// Manages members, superior-subordinate relationships, and the command chain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrganizationHierarchy {
    /// All members in the organization
    members: HashMap<MemberId, Member>,

    /// Superior -> List of direct subordinates
    reporting_lines: HashMap<MemberId, Vec<MemberId>>,

    /// Top of the hierarchy (supreme commander)
    supreme_commander: Option<MemberId>,
}

impl OrganizationHierarchy {
    /// Create a new empty organization hierarchy
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a member to the organization
    ///
    /// If the member has no superior, they become the supreme commander.
    /// Updates reporting lines automatically.
    pub fn add_member(&mut self, member: Member) {
        let id = member.id.clone();
        let superior = member.superior.clone();

        self.members.insert(id.clone(), member);

        // Update reporting lines
        if let Some(superior_id) = superior {
            self.reporting_lines
                .entry(superior_id)
                .or_default()
                .push(id);
        } else {
            // No superior = supreme commander
            self.supreme_commander = Some(id);
        }
    }

    /// Get a member by ID (immutable)
    pub fn get_member(&self, member_id: &MemberId) -> Option<&Member> {
        self.members.get(member_id)
    }

    /// Get a member by ID (mutable)
    pub fn get_member_mut(&mut self, member_id: &MemberId) -> Option<&mut Member> {
        self.members.get_mut(member_id)
    }

    /// Remove a member from the organization
    ///
    /// Also removes them from reporting lines and reassigns supreme commander if needed.
    pub fn remove_member(&mut self, member_id: &MemberId) -> Option<Member> {
        let member = self.members.remove(member_id)?;

        // Remove from superior's reporting line
        if let Some(superior_id) = &member.superior {
            if let Some(subordinates) = self.reporting_lines.get_mut(superior_id) {
                subordinates.retain(|id| id != member_id);
            }
        }

        // Remove their reporting line
        self.reporting_lines.remove(member_id);

        // If they were supreme commander, clear it
        if self.supreme_commander.as_ref() == Some(member_id) {
            self.supreme_commander = None;
        }

        Some(member)
    }

    /// Get direct subordinates of a member
    pub fn get_subordinates(&self, member_id: &MemberId) -> Vec<&Member> {
        self.reporting_lines
            .get(member_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.members.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all subordinates (direct and indirect) recursively
    pub fn get_all_subordinates(&self, member_id: &MemberId) -> Vec<&Member> {
        let mut all_subordinates = Vec::new();
        let mut to_visit = vec![member_id.clone()];

        while let Some(current_id) = to_visit.pop() {
            if let Some(direct_subs) = self.reporting_lines.get(&current_id) {
                for sub_id in direct_subs {
                    if let Some(member) = self.members.get(sub_id) {
                        all_subordinates.push(member);
                        to_visit.push(sub_id.clone());
                    }
                }
            }
        }

        all_subordinates
    }

    /// Check if one member is a direct subordinate of another
    pub fn is_direct_subordinate(&self, subordinate_id: &MemberId, superior_id: &MemberId) -> bool {
        self.members
            .get(subordinate_id)
            .and_then(|m| m.superior.as_ref())
            .map(|sup| sup == superior_id)
            .unwrap_or(false)
    }

    /// Get the supreme commander
    pub fn get_supreme_commander(&self) -> Option<&Member> {
        self.supreme_commander
            .as_ref()
            .and_then(|id| self.members.get(id))
    }

    /// Get all members
    pub fn all_members(&self) -> impl Iterator<Item = (&MemberId, &Member)> {
        self.members.iter()
    }

    /// Get number of members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Check if a member exists
    pub fn has_member(&self, member_id: &MemberId) -> bool {
        self.members.contains_key(member_id)
    }

    /// Get chain depth (distance from supreme commander)
    pub fn get_chain_depth(&self, member_id: &MemberId) -> Option<u32> {
        if !self.has_member(member_id) {
            return None;
        }

        let mut depth = 0;
        let mut current_id = member_id.clone();

        while let Some(member) = self.members.get(&current_id) {
            if let Some(superior_id) = &member.superior {
                depth += 1;
                current_id = superior_id.clone();
            } else {
                break;
            }
        }

        Some(depth)
    }

    /// Clear all members
    pub fn clear(&mut self) {
        self.members.clear();
        self.reporting_lines.clear();
        self.supreme_commander = None;
    }
}

/// Global hierarchy state for all factions (Runtime State)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HierarchyState {
    faction_hierarchies: HashMap<FactionId, OrganizationHierarchy>,
}

impl HierarchyState {
    /// Create a new empty hierarchy state
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new faction (creates empty hierarchy)
    pub fn register_faction(&mut self, faction_id: impl Into<String>) {
        self.faction_hierarchies
            .entry(faction_id.into())
            .or_default();
    }

    /// Get a faction's hierarchy (immutable)
    pub fn get_hierarchy(&self, faction_id: &FactionId) -> Option<&OrganizationHierarchy> {
        self.faction_hierarchies.get(faction_id)
    }

    /// Get a faction's hierarchy (mutable)
    pub fn get_hierarchy_mut(&mut self, faction_id: &FactionId) -> Option<&mut OrganizationHierarchy> {
        self.faction_hierarchies.get_mut(faction_id)
    }

    /// Check if a faction is registered
    pub fn has_faction(&self, faction_id: &FactionId) -> bool {
        self.faction_hierarchies.contains_key(faction_id)
    }

    /// Get all faction hierarchies (immutable)
    pub fn all_hierarchies(&self) -> impl Iterator<Item = (&FactionId, &OrganizationHierarchy)> {
        self.faction_hierarchies.iter()
    }

    /// Get all faction hierarchies (mutable)
    pub fn all_hierarchies_mut(&mut self) -> impl Iterator<Item = (&FactionId, &mut OrganizationHierarchy)> {
        self.faction_hierarchies.iter_mut()
    }

    /// Get number of registered factions
    pub fn faction_count(&self) -> usize {
        self.faction_hierarchies.len()
    }

    /// Remove a faction and its entire hierarchy
    pub fn remove_faction(&mut self, faction_id: &FactionId) -> Option<OrganizationHierarchy> {
        self.faction_hierarchies.remove(faction_id)
    }

    /// Get total member count across all factions
    pub fn total_member_count(&self) -> usize {
        self.faction_hierarchies
            .values()
            .map(|h| h.member_count())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_member(id: &str, rank: &str, superior: Option<&str>) -> Member {
        let mut member = Member::new(id, format!("Member {}", id), rank);
        if let Some(sup) = superior {
            member = member.with_superior(sup);
        }
        member
    }

    #[test]
    fn test_organization_hierarchy_creation() {
        let hierarchy = OrganizationHierarchy::new();
        assert_eq!(hierarchy.member_count(), 0);
        assert!(hierarchy.get_supreme_commander().is_none());
    }

    #[test]
    fn test_add_member_without_superior() {
        let mut hierarchy = OrganizationHierarchy::new();
        let commander = create_test_member("commander", "general", None);

        hierarchy.add_member(commander);

        assert_eq!(hierarchy.member_count(), 1);
        assert!(hierarchy.get_supreme_commander().is_some());
        assert_eq!(hierarchy.get_supreme_commander().unwrap().id, "commander");
    }

    #[test]
    fn test_add_member_with_superior() {
        let mut hierarchy = OrganizationHierarchy::new();
        let commander = create_test_member("commander", "general", None);
        let captain = create_test_member("captain1", "captain", Some("commander"));

        hierarchy.add_member(commander);
        hierarchy.add_member(captain);

        assert_eq!(hierarchy.member_count(), 2);
        assert!(hierarchy.get_member(&"captain1".to_string()).is_some());
    }

    #[test]
    fn test_get_subordinates() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));
        hierarchy.add_member(create_test_member("captain1", "captain", Some("commander")));
        hierarchy.add_member(create_test_member("captain2", "captain", Some("commander")));

        let subs = hierarchy.get_subordinates(&"commander".to_string());
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn test_get_all_subordinates_recursive() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));
        hierarchy.add_member(create_test_member("captain1", "captain", Some("commander")));
        hierarchy.add_member(create_test_member("sergeant1", "sergeant", Some("captain1")));
        hierarchy.add_member(create_test_member("private1", "private", Some("sergeant1")));

        let all_subs = hierarchy.get_all_subordinates(&"commander".to_string());
        assert_eq!(all_subs.len(), 3); // captain1, sergeant1, private1
    }

    #[test]
    fn test_is_direct_subordinate() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));
        hierarchy.add_member(create_test_member("captain1", "captain", Some("commander")));

        assert!(hierarchy.is_direct_subordinate(&"captain1".to_string(), &"commander".to_string()));
        assert!(!hierarchy.is_direct_subordinate(&"commander".to_string(), &"captain1".to_string()));
    }

    #[test]
    fn test_get_chain_depth() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));
        hierarchy.add_member(create_test_member("captain1", "captain", Some("commander")));
        hierarchy.add_member(create_test_member("sergeant1", "sergeant", Some("captain1")));

        assert_eq!(hierarchy.get_chain_depth(&"commander".to_string()), Some(0));
        assert_eq!(hierarchy.get_chain_depth(&"captain1".to_string()), Some(1));
        assert_eq!(hierarchy.get_chain_depth(&"sergeant1".to_string()), Some(2));
    }

    #[test]
    fn test_remove_member() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));
        hierarchy.add_member(create_test_member("captain1", "captain", Some("commander")));

        let removed = hierarchy.remove_member(&"captain1".to_string());
        assert!(removed.is_some());
        assert_eq!(hierarchy.member_count(), 1);
        assert_eq!(hierarchy.get_subordinates(&"commander".to_string()).len(), 0);
    }

    #[test]
    fn test_remove_supreme_commander() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));

        hierarchy.remove_member(&"commander".to_string());
        assert!(hierarchy.get_supreme_commander().is_none());
        assert_eq!(hierarchy.member_count(), 0);
    }

    #[test]
    fn test_hierarchy_state_creation() {
        let state = HierarchyState::new();
        assert_eq!(state.faction_count(), 0);
    }

    #[test]
    fn test_register_faction() {
        let mut state = HierarchyState::new();
        state.register_faction("faction_a");

        assert_eq!(state.faction_count(), 1);
        assert!(state.has_faction(&"faction_a".to_string()));
    }

    #[test]
    fn test_get_hierarchy() {
        let mut state = HierarchyState::new();
        state.register_faction("faction_a");

        let hierarchy = state.get_hierarchy(&"faction_a".to_string());
        assert!(hierarchy.is_some());
        assert_eq!(hierarchy.unwrap().member_count(), 0);
    }

    #[test]
    fn test_get_hierarchy_mut() {
        let mut state = HierarchyState::new();
        state.register_faction("faction_a");

        if let Some(hierarchy) = state.get_hierarchy_mut(&"faction_a".to_string()) {
            hierarchy.add_member(create_test_member("m1", "private", None));
        }

        let hierarchy = state.get_hierarchy(&"faction_a".to_string()).unwrap();
        assert_eq!(hierarchy.member_count(), 1);
    }

    #[test]
    fn test_remove_faction() {
        let mut state = HierarchyState::new();
        state.register_faction("faction_a");
        state.register_faction("faction_b");

        let removed = state.remove_faction(&"faction_a".to_string());
        assert!(removed.is_some());
        assert_eq!(state.faction_count(), 1);
        assert!(!state.has_faction(&"faction_a".to_string()));
    }

    #[test]
    fn test_total_member_count() {
        let mut state = HierarchyState::new();
        state.register_faction("faction_a");
        state.register_faction("faction_b");

        if let Some(h) = state.get_hierarchy_mut(&"faction_a".to_string()) {
            h.add_member(create_test_member("m1", "private", None));
            h.add_member(create_test_member("m2", "private", None));
        }

        if let Some(h) = state.get_hierarchy_mut(&"faction_b".to_string()) {
            h.add_member(create_test_member("m3", "private", None));
        }

        assert_eq!(state.total_member_count(), 3);
    }

    #[test]
    fn test_serialization() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("commander", "general", None));

        let json = serde_json::to_string(&hierarchy).unwrap();
        let deserialized: OrganizationHierarchy = serde_json::from_str(&json).unwrap();

        assert_eq!(hierarchy.member_count(), deserialized.member_count());
    }

    #[test]
    fn test_clear_hierarchy() {
        let mut hierarchy = OrganizationHierarchy::new();
        hierarchy.add_member(create_test_member("m1", "private", None));
        hierarchy.add_member(create_test_member("m2", "private", None));

        hierarchy.clear();
        assert_eq!(hierarchy.member_count(), 0);
        assert!(hierarchy.get_supreme_commander().is_none());
    }
}
