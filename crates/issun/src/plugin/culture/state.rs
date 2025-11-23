//! State management for CulturePlugin
//!
//! Provides OrganizationCulture and CultureState for managing
//! organizational culture across multiple factions.

use super::types::{CultureTag, FactionId, Member, MemberId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Culture structure for a single organization
///
/// Manages members, culture tags, and cultural alignment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrganizationCulture {
    /// All members in the organization
    members: HashMap<MemberId, Member>,

    /// Culture tags defining the organization's "atmosphere"
    culture_tags: HashSet<CultureTag>,

    /// Culture strength multiplier (0.0-2.0)
    /// Overrides global config for this specific organization
    culture_strength: Option<f32>,
}

impl OrganizationCulture {
    /// Create a new empty organization culture
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a member to the organization
    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.id.clone(), member);
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
    pub fn remove_member(&mut self, member_id: &MemberId) -> Option<Member> {
        self.members.remove(member_id)
    }

    /// Check if a member exists
    pub fn has_member(&self, member_id: &MemberId) -> bool {
        self.members.contains_key(member_id)
    }

    /// Get all members
    pub fn all_members(&self) -> impl Iterator<Item = (&MemberId, &Member)> {
        self.members.iter()
    }

    /// Get all members (mutable)
    pub fn all_members_mut(&mut self) -> impl Iterator<Item = (&MemberId, &mut Member)> {
        self.members.iter_mut()
    }

    /// Get number of members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Add a culture tag
    pub fn add_culture_tag(&mut self, tag: CultureTag) {
        self.culture_tags.insert(tag);
    }

    /// Remove a culture tag
    pub fn remove_culture_tag(&mut self, tag: &CultureTag) -> bool {
        self.culture_tags.remove(tag)
    }

    /// Check if organization has a specific culture tag
    pub fn has_culture_tag(&self, tag: &CultureTag) -> bool {
        self.culture_tags.contains(tag)
    }

    /// Get all culture tags
    pub fn culture_tags(&self) -> &HashSet<CultureTag> {
        &self.culture_tags
    }

    /// Get number of culture tags
    pub fn culture_tag_count(&self) -> usize {
        self.culture_tags.len()
    }

    /// Set culture strength for this organization
    pub fn set_culture_strength(&mut self, strength: f32) {
        self.culture_strength = Some(strength.clamp(0.0, 2.0));
    }

    /// Get culture strength (returns None if using global config)
    pub fn culture_strength(&self) -> Option<f32> {
        self.culture_strength
    }

    /// Clear culture strength override (use global config)
    pub fn clear_culture_strength(&mut self) {
        self.culture_strength = None;
    }

    /// Calculate average stress level across all members
    pub fn average_stress(&self) -> f32 {
        if self.members.is_empty() {
            return 0.0;
        }

        let total_stress: f32 = self.members.values().map(|m| m.stress).sum();
        total_stress / self.members.len() as f32
    }

    /// Calculate average fervor level across all members
    pub fn average_fervor(&self) -> f32 {
        if self.members.is_empty() {
            return 0.0;
        }

        let total_fervor: f32 = self.members.values().map(|m| m.fervor).sum();
        total_fervor / self.members.len() as f32
    }

    /// Get members with high stress (above threshold)
    pub fn high_stress_members(&self, threshold: f32) -> Vec<&Member> {
        self.members
            .values()
            .filter(|m| m.stress >= threshold)
            .collect()
    }

    /// Get members with high fervor (above threshold)
    pub fn high_fervor_members(&self, threshold: f32) -> Vec<&Member> {
        self.members
            .values()
            .filter(|m| m.fervor >= threshold)
            .collect()
    }

    /// Clear all members
    pub fn clear_members(&mut self) {
        self.members.clear();
    }

    /// Clear all culture tags
    pub fn clear_culture_tags(&mut self) {
        self.culture_tags.clear();
    }

    /// Clear everything
    pub fn clear(&mut self) {
        self.members.clear();
        self.culture_tags.clear();
        self.culture_strength = None;
    }
}

/// Global culture state for all factions (Runtime State)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CultureState {
    faction_cultures: HashMap<FactionId, OrganizationCulture>,
}

impl CultureState {
    /// Create a new empty culture state
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new faction (creates empty culture)
    pub fn register_faction(&mut self, faction_id: impl Into<String>) {
        self.faction_cultures.entry(faction_id.into()).or_default();
    }

    /// Get a faction's culture (immutable)
    pub fn get_culture(&self, faction_id: &FactionId) -> Option<&OrganizationCulture> {
        self.faction_cultures.get(faction_id)
    }

    /// Get a faction's culture (mutable)
    pub fn get_culture_mut(&mut self, faction_id: &FactionId) -> Option<&mut OrganizationCulture> {
        self.faction_cultures.get_mut(faction_id)
    }

    /// Check if a faction is registered
    pub fn has_faction(&self, faction_id: &FactionId) -> bool {
        self.faction_cultures.contains_key(faction_id)
    }

    /// Get all faction cultures (immutable)
    pub fn all_cultures(&self) -> impl Iterator<Item = (&FactionId, &OrganizationCulture)> {
        self.faction_cultures.iter()
    }

    /// Get all faction cultures (mutable)
    pub fn all_cultures_mut(
        &mut self,
    ) -> impl Iterator<Item = (&FactionId, &mut OrganizationCulture)> {
        self.faction_cultures.iter_mut()
    }

    /// Get number of registered factions
    pub fn faction_count(&self) -> usize {
        self.faction_cultures.len()
    }

    /// Remove a faction and its entire culture
    pub fn remove_faction(&mut self, faction_id: &FactionId) -> Option<OrganizationCulture> {
        self.faction_cultures.remove(faction_id)
    }

    /// Get total member count across all factions
    pub fn total_member_count(&self) -> usize {
        self.faction_cultures
            .values()
            .map(|c| c.member_count())
            .sum()
    }

    /// Get global average stress across all factions
    pub fn global_average_stress(&self) -> f32 {
        if self.faction_cultures.is_empty() {
            return 0.0;
        }

        let total_stress: f32 = self
            .faction_cultures
            .values()
            .map(|c| c.average_stress())
            .sum();
        total_stress / self.faction_cultures.len() as f32
    }

    /// Get global average fervor across all factions
    pub fn global_average_fervor(&self) -> f32 {
        if self.faction_cultures.is_empty() {
            return 0.0;
        }

        let total_fervor: f32 = self
            .faction_cultures
            .values()
            .map(|c| c.average_fervor())
            .sum();
        total_fervor / self.faction_cultures.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_member(id: &str, name: &str) -> Member {
        Member::new(id, name)
    }

    fn create_member_with_stress(id: &str, name: &str, stress: f32) -> Member {
        Member::new(id, name).with_stress(stress)
    }

    fn create_member_with_fervor(id: &str, name: &str, fervor: f32) -> Member {
        Member::new(id, name).with_fervor(fervor)
    }

    #[test]
    fn test_organization_culture_creation() {
        let culture = OrganizationCulture::new();
        assert_eq!(culture.member_count(), 0);
        assert_eq!(culture.culture_tag_count(), 0);
    }

    #[test]
    fn test_add_member() {
        let mut culture = OrganizationCulture::new();
        let member = create_test_member("m1", "Alice");

        culture.add_member(member);

        assert_eq!(culture.member_count(), 1);
        assert!(culture.has_member(&"m1".to_string()));
    }

    #[test]
    fn test_get_member() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_test_member("m1", "Alice"));

        let member = culture.get_member(&"m1".to_string());
        assert!(member.is_some());
        assert_eq!(member.unwrap().name, "Alice");
    }

    #[test]
    fn test_get_member_mut() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_test_member("m1", "Alice"));

        if let Some(member) = culture.get_member_mut(&"m1".to_string()) {
            member.stress = 0.7;
        }

        let member = culture.get_member(&"m1".to_string()).unwrap();
        assert_eq!(member.stress, 0.7);
    }

    #[test]
    fn test_remove_member() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_test_member("m1", "Alice"));

        let removed = culture.remove_member(&"m1".to_string());
        assert!(removed.is_some());
        assert_eq!(culture.member_count(), 0);
    }

    #[test]
    fn test_add_culture_tag() {
        let mut culture = OrganizationCulture::new();
        culture.add_culture_tag(CultureTag::RiskTaking);

        assert_eq!(culture.culture_tag_count(), 1);
        assert!(culture.has_culture_tag(&CultureTag::RiskTaking));
    }

    #[test]
    fn test_remove_culture_tag() {
        let mut culture = OrganizationCulture::new();
        culture.add_culture_tag(CultureTag::RiskTaking);

        let removed = culture.remove_culture_tag(&CultureTag::RiskTaking);
        assert!(removed);
        assert_eq!(culture.culture_tag_count(), 0);
    }

    #[test]
    fn test_multiple_culture_tags() {
        let mut culture = OrganizationCulture::new();
        culture.add_culture_tag(CultureTag::RiskTaking);
        culture.add_culture_tag(CultureTag::PsychologicalSafety);
        culture.add_culture_tag(CultureTag::Fanatic);

        assert_eq!(culture.culture_tag_count(), 3);
    }

    #[test]
    fn test_culture_strength() {
        let mut culture = OrganizationCulture::new();

        assert!(culture.culture_strength().is_none());

        culture.set_culture_strength(1.5);
        assert_eq!(culture.culture_strength(), Some(1.5));

        culture.clear_culture_strength();
        assert!(culture.culture_strength().is_none());
    }

    #[test]
    fn test_culture_strength_clamping() {
        let mut culture = OrganizationCulture::new();

        culture.set_culture_strength(3.0); // Should clamp to 2.0
        assert_eq!(culture.culture_strength(), Some(2.0));

        culture.set_culture_strength(-1.0); // Should clamp to 0.0
        assert_eq!(culture.culture_strength(), Some(0.0));
    }

    #[test]
    fn test_average_stress() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_member_with_stress("m1", "Alice", 0.3));
        culture.add_member(create_member_with_stress("m2", "Bob", 0.7));

        assert_eq!(culture.average_stress(), 0.5);
    }

    #[test]
    fn test_average_stress_empty() {
        let culture = OrganizationCulture::new();
        assert_eq!(culture.average_stress(), 0.0);
    }

    #[test]
    fn test_average_fervor() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_member_with_fervor("m1", "Alice", 0.4));
        culture.add_member(create_member_with_fervor("m2", "Bob", 0.8));

        assert_eq!(culture.average_fervor(), 0.6);
    }

    #[test]
    fn test_high_stress_members() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_member_with_stress("m1", "Alice", 0.3));
        culture.add_member(create_member_with_stress("m2", "Bob", 0.7));
        culture.add_member(create_member_with_stress("m3", "Charlie", 0.9));

        let high_stress = culture.high_stress_members(0.6);
        assert_eq!(high_stress.len(), 2); // Bob and Charlie
    }

    #[test]
    fn test_high_fervor_members() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_member_with_fervor("m1", "Alice", 0.3));
        culture.add_member(create_member_with_fervor("m2", "Bob", 0.7));
        culture.add_member(create_member_with_fervor("m3", "Charlie", 0.95));

        let high_fervor = culture.high_fervor_members(0.9);
        assert_eq!(high_fervor.len(), 1); // Charlie only
    }

    #[test]
    fn test_clear() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_test_member("m1", "Alice"));
        culture.add_culture_tag(CultureTag::RiskTaking);
        culture.set_culture_strength(1.5);

        culture.clear();

        assert_eq!(culture.member_count(), 0);
        assert_eq!(culture.culture_tag_count(), 0);
        assert!(culture.culture_strength().is_none());
    }

    #[test]
    fn test_culture_state_creation() {
        let state = CultureState::new();
        assert_eq!(state.faction_count(), 0);
    }

    #[test]
    fn test_register_faction() {
        let mut state = CultureState::new();
        state.register_faction("faction_a");

        assert_eq!(state.faction_count(), 1);
        assert!(state.has_faction(&"faction_a".to_string()));
    }

    #[test]
    fn test_get_culture() {
        let mut state = CultureState::new();
        state.register_faction("faction_a");

        let culture = state.get_culture(&"faction_a".to_string());
        assert!(culture.is_some());
        assert_eq!(culture.unwrap().member_count(), 0);
    }

    #[test]
    fn test_get_culture_mut() {
        let mut state = CultureState::new();
        state.register_faction("faction_a");

        if let Some(culture) = state.get_culture_mut(&"faction_a".to_string()) {
            culture.add_member(create_test_member("m1", "Alice"));
        }

        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        assert_eq!(culture.member_count(), 1);
    }

    #[test]
    fn test_remove_faction() {
        let mut state = CultureState::new();
        state.register_faction("faction_a");
        state.register_faction("faction_b");

        let removed = state.remove_faction(&"faction_a".to_string());
        assert!(removed.is_some());
        assert_eq!(state.faction_count(), 1);
        assert!(!state.has_faction(&"faction_a".to_string()));
    }

    #[test]
    fn test_total_member_count() {
        let mut state = CultureState::new();
        state.register_faction("faction_a");
        state.register_faction("faction_b");

        if let Some(c) = state.get_culture_mut(&"faction_a".to_string()) {
            c.add_member(create_test_member("m1", "Alice"));
            c.add_member(create_test_member("m2", "Bob"));
        }

        if let Some(c) = state.get_culture_mut(&"faction_b".to_string()) {
            c.add_member(create_test_member("m3", "Charlie"));
        }

        assert_eq!(state.total_member_count(), 3);
    }

    #[test]
    fn test_global_average_stress() {
        let mut state = CultureState::new();
        state.register_faction("faction_a");
        state.register_faction("faction_b");

        if let Some(c) = state.get_culture_mut(&"faction_a".to_string()) {
            c.add_member(create_member_with_stress("m1", "Alice", 0.4));
        }

        if let Some(c) = state.get_culture_mut(&"faction_b".to_string()) {
            c.add_member(create_member_with_stress("m2", "Bob", 0.8));
        }

        // Average of faction averages: (0.4 + 0.8) / 2 = 0.6
        assert_eq!(state.global_average_stress(), 0.6);
    }

    #[test]
    fn test_serialization() {
        let mut culture = OrganizationCulture::new();
        culture.add_member(create_test_member("m1", "Alice"));
        culture.add_culture_tag(CultureTag::RiskTaking);

        let json = serde_json::to_string(&culture).unwrap();
        let deserialized: OrganizationCulture = serde_json::from_str(&json).unwrap();

        assert_eq!(culture.member_count(), deserialized.member_count());
        assert_eq!(
            culture.culture_tag_count(),
            deserialized.culture_tag_count()
        );
    }
}
