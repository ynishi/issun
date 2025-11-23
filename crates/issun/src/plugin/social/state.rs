//! State management for SocialPlugin
//!
//! Provides SocialNetwork and SocialMember for managing
//! social networks, influence graphs, and factions across multiple organizations.

use super::types::{
    Faction, FactionId, MemberId, RelationType, SocialCapital, SocialError,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Social member - individual with social capital and network position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialMember {
    pub id: MemberId,
    pub name: String,

    /// Social capital (influence, favors, secrets)
    pub capital: SocialCapital,

    /// Faction memberships (can belong to multiple)
    pub faction_memberships: HashSet<FactionId>,

    /// Perceived network - this member's view of other relationships
    /// (Can be inaccurate - basis for political mistakes)
    pub perceived_network: HashMap<MemberId, HashMap<MemberId, RelationType>>,

    /// Political skill (0.0-1.0) - ability to navigate social dynamics
    pub political_skill: f32,
}

impl SocialMember {
    /// Create a new social member
    pub fn new(id: MemberId, name: String) -> Self {
        Self {
            id,
            name,
            capital: SocialCapital::default(),
            faction_memberships: HashSet::new(),
            perceived_network: HashMap::new(),
            political_skill: 0.5,
        }
    }

    /// Check if member belongs to a faction
    pub fn is_member_of(&self, faction_id: &FactionId) -> bool {
        self.faction_memberships.contains(faction_id)
    }

    /// Add faction membership
    pub fn join_faction(&mut self, faction_id: FactionId) {
        self.faction_memberships.insert(faction_id);
    }

    /// Remove faction membership
    pub fn leave_faction(&mut self, faction_id: &FactionId) -> bool {
        self.faction_memberships.remove(faction_id)
    }

    /// Get number of faction memberships
    pub fn faction_count(&self) -> usize {
        self.faction_memberships.len()
    }
}

/// Social network for a single organization
///
/// Manages members, relationships, factions, and network analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialNetwork {
    /// All members in the network
    members: HashMap<MemberId, SocialMember>,

    /// Relationship graph (from, to) -> [RelationType]
    /// Allows multiple relationship types between same pair
    relations: HashMap<(MemberId, MemberId), Vec<RelationType>>,

    /// Factions (political coalitions)
    factions: HashMap<FactionId, Faction>,

    /// Centrality cache (updated periodically)
    /// Stores last computed centrality metrics for performance
    centrality_cache_valid: bool,

    /// Last turn when centrality was calculated
    last_centrality_update: u64,
}

impl SocialNetwork {
    /// Create a new empty social network
    pub fn new() -> Self {
        Self::default()
    }

    // ===== Member Management =====

    /// Add a member to the network
    pub fn add_member(&mut self, member: SocialMember) {
        self.members.insert(member.id.clone(), member);
        self.invalidate_centrality_cache();
    }

    /// Get a member by ID (immutable)
    pub fn get_member(&self, member_id: &MemberId) -> Option<&SocialMember> {
        self.members.get(member_id)
    }

    /// Get a member by ID (mutable)
    pub fn get_member_mut(&mut self, member_id: &MemberId) -> Option<&mut SocialMember> {
        self.members.get_mut(member_id)
    }

    /// Remove a member from the network
    pub fn remove_member(&mut self, member_id: &MemberId) -> Option<SocialMember> {
        // Remove from factions
        if let Some(member) = self.members.get(member_id) {
            for faction_id in member.faction_memberships.clone() {
                if let Some(faction) = self.factions.get_mut(&faction_id) {
                    faction.remove_member(member_id);
                }
            }
        }

        // Remove all relations involving this member
        self.relations
            .retain(|(from, to), _| from != member_id && to != member_id);

        self.invalidate_centrality_cache();
        self.members.remove(member_id)
    }

    /// Check if a member exists
    pub fn has_member(&self, member_id: &MemberId) -> bool {
        self.members.contains_key(member_id)
    }

    /// Get all members
    pub fn all_members(&self) -> impl Iterator<Item = (&MemberId, &SocialMember)> {
        self.members.iter()
    }

    /// Get all members (mutable)
    pub fn all_members_mut(&mut self) -> impl Iterator<Item = (&MemberId, &mut SocialMember)> {
        self.members.iter_mut()
    }

    /// Get number of members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    // ===== Relationship Management =====

    /// Add a relationship between two members
    pub fn add_relation(
        &mut self,
        from: MemberId,
        to: MemberId,
        relation: RelationType,
    ) -> Result<(), SocialError> {
        if !self.has_member(&from) {
            return Err(SocialError::MemberNotFound(from));
        }
        if !self.has_member(&to) {
            return Err(SocialError::MemberNotFound(to));
        }

        self.relations
            .entry((from.clone(), to.clone()))
            .or_insert_with(Vec::new)
            .push(relation);

        self.invalidate_centrality_cache();
        Ok(())
    }

    /// Get all relations between two members
    pub fn get_relations(
        &self,
        from: &MemberId,
        to: &MemberId,
    ) -> Option<&Vec<RelationType>> {
        self.relations.get(&(from.clone(), to.clone()))
    }

    /// Remove a specific relation type between two members
    pub fn remove_relation(
        &mut self,
        from: &MemberId,
        to: &MemberId,
        relation_type_name: &str,
    ) -> bool {
        if let Some(relations) = self.relations.get_mut(&(from.clone(), to.clone())) {
            let before_len = relations.len();
            relations.retain(|r| !r.description().contains(relation_type_name));
            let removed = before_len != relations.len();

            if removed {
                self.invalidate_centrality_cache();
            }

            return removed;
        }
        false
    }

    /// Get all relations in the network
    pub fn all_relations(
        &self,
    ) -> impl Iterator<Item = (&(MemberId, MemberId), &Vec<RelationType>)> {
        self.relations.iter()
    }

    /// Get relations count
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    /// Get neighbors (members with direct connections)
    pub fn get_neighbors(&self, member_id: &MemberId) -> Vec<MemberId> {
        let mut neighbors = HashSet::new();

        for ((from, to), _) in &self.relations {
            if from == member_id {
                neighbors.insert(to.clone());
            }
            if to == member_id {
                neighbors.insert(from.clone());
            }
        }

        neighbors.into_iter().collect()
    }

    // ===== Faction Management =====

    /// Add a faction
    pub fn add_faction(&mut self, faction: Faction) {
        self.factions.insert(faction.id.clone(), faction);
    }

    /// Get a faction by ID (immutable)
    pub fn get_faction(&self, faction_id: &FactionId) -> Option<&Faction> {
        self.factions.get(faction_id)
    }

    /// Get a faction by ID (mutable)
    pub fn get_faction_mut(&mut self, faction_id: &FactionId) -> Option<&mut Faction> {
        self.factions.get_mut(faction_id)
    }

    /// Remove a faction
    pub fn remove_faction(&mut self, faction_id: &FactionId) -> Option<Faction> {
        // Remove faction membership from all members
        for member in self.members.values_mut() {
            member.leave_faction(faction_id);
        }

        self.factions.remove(faction_id)
    }

    /// Get all factions
    pub fn all_factions(&self) -> impl Iterator<Item = (&FactionId, &Faction)> {
        self.factions.iter()
    }

    /// Get all factions (mutable)
    pub fn all_factions_mut(&mut self) -> impl Iterator<Item = (&FactionId, &mut Faction)> {
        self.factions.iter_mut()
    }

    /// Get faction count
    pub fn faction_count(&self) -> usize {
        self.factions.len()
    }

    // ===== Centrality Cache Management =====

    /// Invalidate centrality cache (call when network structure changes)
    pub fn invalidate_centrality_cache(&mut self) {
        self.centrality_cache_valid = false;
    }

    /// Check if centrality cache is valid
    pub fn is_centrality_cache_valid(&self) -> bool {
        self.centrality_cache_valid
    }

    /// Mark centrality cache as valid (call after recalculation)
    pub fn mark_centrality_cache_valid(&mut self, current_turn: u64) {
        self.centrality_cache_valid = true;
        self.last_centrality_update = current_turn;
    }

    /// Get last centrality update turn
    pub fn last_centrality_update(&self) -> u64 {
        self.last_centrality_update
    }

    // ===== Analysis Helpers =====

    /// Check if network is operational (has sufficient members and connections)
    pub fn is_operational(&self) -> bool {
        self.member_count() >= 2 && self.relation_count() > 0
    }

    /// Calculate graph connectivity (simple metric: relations / possible_relations)
    pub fn calculate_graph_connectivity(&self) -> f32 {
        let n = self.member_count();
        if n < 2 {
            return 0.0;
        }

        let possible_relations = n * (n - 1);
        let actual_relations = self.relation_count();

        actual_relations as f32 / possible_relations as f32
    }
}

/// Global state for SocialPlugin across all factions
///
/// This is the main state container registered as a resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialState {
    /// Social networks indexed by faction ID
    networks: HashMap<FactionId, SocialNetwork>,
}

impl crate::resources::Resource for SocialState {}

impl SocialState {
    /// Create a new empty social state
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a faction (create empty network)
    pub fn register_faction(&mut self, faction_id: FactionId) {
        self.networks
            .entry(faction_id)
            .or_insert_with(SocialNetwork::new);
    }

    /// Unregister a faction (remove network)
    pub fn unregister_faction(&mut self, faction_id: &FactionId) -> Option<SocialNetwork> {
        self.networks.remove(faction_id)
    }

    /// Get a network by faction ID (immutable)
    pub fn get_network(&self, faction_id: &FactionId) -> Option<&SocialNetwork> {
        self.networks.get(faction_id)
    }

    /// Get a network by faction ID (mutable)
    pub fn get_network_mut(&mut self, faction_id: &FactionId) -> Option<&mut SocialNetwork> {
        self.networks.get_mut(faction_id)
    }

    /// Get all networks
    pub fn all_networks(&self) -> impl Iterator<Item = (&FactionId, &SocialNetwork)> {
        self.networks.iter()
    }

    /// Get all networks (mutable)
    pub fn all_networks_mut(&mut self) -> impl Iterator<Item = (&FactionId, &mut SocialNetwork)> {
        self.networks.iter_mut()
    }

    /// Get number of registered factions
    pub fn faction_count(&self) -> usize {
        self.networks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_member_new() {
        let member = SocialMember::new("m1".to_string(), "Alice".to_string());
        assert_eq!(member.id, "m1");
        assert_eq!(member.name, "Alice");
        assert_eq!(member.political_skill, 0.5);
    }

    #[test]
    fn test_social_member_factions() {
        let mut member = SocialMember::new("m1".to_string(), "Alice".to_string());

        member.join_faction("faction1".to_string());
        assert!(member.is_member_of(&"faction1".to_string()));
        assert_eq!(member.faction_count(), 1);

        member.leave_faction(&"faction1".to_string());
        assert!(!member.is_member_of(&"faction1".to_string()));
        assert_eq!(member.faction_count(), 0);
    }

    #[test]
    fn test_social_network_new() {
        let network = SocialNetwork::new();
        assert_eq!(network.member_count(), 0);
        assert_eq!(network.faction_count(), 0);
    }

    #[test]
    fn test_social_network_add_member() {
        let mut network = SocialNetwork::new();
        let member = SocialMember::new("m1".to_string(), "Alice".to_string());

        network.add_member(member);
        assert_eq!(network.member_count(), 1);
        assert!(network.has_member(&"m1".to_string()));
    }

    #[test]
    fn test_social_network_remove_member() {
        let mut network = SocialNetwork::new();
        let member = SocialMember::new("m1".to_string(), "Alice".to_string());

        network.add_member(member);
        let removed = network.remove_member(&"m1".to_string());

        assert!(removed.is_some());
        assert_eq!(network.member_count(), 0);
    }

    #[test]
    fn test_social_network_add_relation() {
        let mut network = SocialNetwork::new();
        network.add_member(SocialMember::new("m1".to_string(), "Alice".to_string()));
        network.add_member(SocialMember::new("m2".to_string(), "Bob".to_string()));

        let result = network.add_relation(
            "m1".to_string(),
            "m2".to_string(),
            RelationType::Trust { strength: 0.8 },
        );

        assert!(result.is_ok());
        assert_eq!(network.relation_count(), 1);
    }

    #[test]
    fn test_social_network_add_relation_invalid_member() {
        let mut network = SocialNetwork::new();

        let result = network.add_relation(
            "m1".to_string(),
            "m2".to_string(),
            RelationType::Trust { strength: 0.8 },
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_social_network_get_neighbors() {
        let mut network = SocialNetwork::new();
        network.add_member(SocialMember::new("m1".to_string(), "Alice".to_string()));
        network.add_member(SocialMember::new("m2".to_string(), "Bob".to_string()));
        network.add_member(SocialMember::new("m3".to_string(), "Carol".to_string()));

        network
            .add_relation(
                "m1".to_string(),
                "m2".to_string(),
                RelationType::Trust { strength: 0.8 },
            )
            .unwrap();
        network
            .add_relation(
                "m1".to_string(),
                "m3".to_string(),
                RelationType::Trust { strength: 0.5 },
            )
            .unwrap();

        let neighbors = network.get_neighbors(&"m1".to_string());
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&"m2".to_string()));
        assert!(neighbors.contains(&"m3".to_string()));
    }

    #[test]
    fn test_social_network_factions() {
        let mut network = SocialNetwork::new();
        let faction = Faction::new("f1".to_string(), "Test Faction".to_string());

        network.add_faction(faction);
        assert_eq!(network.faction_count(), 1);
        assert!(network.get_faction(&"f1".to_string()).is_some());
    }

    #[test]
    fn test_social_network_connectivity() {
        let mut network = SocialNetwork::new();
        network.add_member(SocialMember::new("m1".to_string(), "Alice".to_string()));
        network.add_member(SocialMember::new("m2".to_string(), "Bob".to_string()));

        // No relations yet
        assert_eq!(network.calculate_graph_connectivity(), 0.0);

        // Add one relation
        network
            .add_relation(
                "m1".to_string(),
                "m2".to_string(),
                RelationType::Trust { strength: 0.8 },
            )
            .unwrap();

        // 1 relation out of 2 possible (m1->m2, m2->m1)
        assert!(network.calculate_graph_connectivity() > 0.0);
    }

    #[test]
    fn test_social_network_cache_invalidation() {
        let mut network = SocialNetwork::new();
        assert!(!network.is_centrality_cache_valid());

        network.mark_centrality_cache_valid(100);
        assert!(network.is_centrality_cache_valid());
        assert_eq!(network.last_centrality_update(), 100);

        network.add_member(SocialMember::new("m1".to_string(), "Alice".to_string()));
        assert!(!network.is_centrality_cache_valid());
    }

    #[test]
    fn test_social_state_new() {
        let state = SocialState::new();
        assert_eq!(state.faction_count(), 0);
    }

    #[test]
    fn test_social_state_register_faction() {
        let mut state = SocialState::new();

        state.register_faction("faction1".to_string());
        assert_eq!(state.faction_count(), 1);
        assert!(state.get_network(&"faction1".to_string()).is_some());
    }

    #[test]
    fn test_social_state_unregister_faction() {
        let mut state = SocialState::new();

        state.register_faction("faction1".to_string());
        let removed = state.unregister_faction(&"faction1".to_string());

        assert!(removed.is_some());
        assert_eq!(state.faction_count(), 0);
    }
}
