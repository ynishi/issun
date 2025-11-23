//! Rank definitions and authority levels

use super::types::RankId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authority levels for ranks
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Copy)]
pub enum AuthorityLevel {
    /// No command authority
    Private,

    /// Can command small units (5-10 members)
    SquadLeader,

    /// Can command multiple squads (20-50 members)
    Captain,

    /// Strategic command (100+ members)
    Commander,

    /// Supreme command (entire organization)
    SupremeCommander,
}

impl AuthorityLevel {
    /// Get numeric value for authority level (for comparisons)
    pub fn value(&self) -> u8 {
        match self {
            AuthorityLevel::Private => 0,
            AuthorityLevel::SquadLeader => 1,
            AuthorityLevel::Captain => 2,
            AuthorityLevel::Commander => 3,
            AuthorityLevel::SupremeCommander => 4,
        }
    }

    /// Check if this authority can command another
    pub fn can_command(&self, other: &AuthorityLevel) -> bool {
        self > other
    }
}

/// Rank definition (static configuration)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RankDefinition {
    pub id: RankId,
    pub name: String,

    /// Rank level (0 = lowest, higher = more senior)
    pub level: u32,

    /// Authority level
    pub authority_level: AuthorityLevel,

    /// Maximum direct subordinates
    pub max_direct_subordinates: usize,
}

impl RankDefinition {
    /// Create a new rank definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        level: u32,
        authority_level: AuthorityLevel,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            level,
            authority_level,
            max_direct_subordinates: match authority_level {
                AuthorityLevel::Private => 0,
                AuthorityLevel::SquadLeader => 5,
                AuthorityLevel::Captain => 20,
                AuthorityLevel::Commander => 50,
                AuthorityLevel::SupremeCommander => 100,
            },
        }
    }

    /// Set maximum direct subordinates
    pub fn with_max_subordinates(mut self, max: usize) -> Self {
        self.max_direct_subordinates = max;
        self
    }

    /// Check if this rank can be promoted to another rank
    pub fn can_promote_to(&self, other: &RankDefinition) -> bool {
        other.level == self.level + 1
    }
}

/// Collection of rank definitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankDefinitions {
    ranks: HashMap<RankId, RankDefinition>,
}

impl crate::resources::Resource for RankDefinitions {}

impl RankDefinitions {
    /// Create a new empty rank definitions collection
    pub fn new() -> Self {
        Self {
            ranks: HashMap::new(),
        }
    }

    /// Add a rank definition
    pub fn add(&mut self, rank: RankDefinition) {
        self.ranks.insert(rank.id.clone(), rank);
    }

    /// Get a rank definition by ID
    pub fn get(&self, rank_id: &RankId) -> Option<&RankDefinition> {
        self.ranks.get(rank_id)
    }

    /// Get the next rank in progression
    pub fn get_next_rank(&self, current_rank: &RankId) -> Option<&RankDefinition> {
        let current = self.get(current_rank)?;
        self.ranks.values().find(|r| r.level == current.level + 1)
    }

    /// Get all ranks sorted by level
    pub fn get_all_sorted(&self) -> Vec<&RankDefinition> {
        let mut ranks: Vec<_> = self.ranks.values().collect();
        ranks.sort_by_key(|r| r.level);
        ranks
    }

    /// Get number of ranks
    pub fn len(&self) -> usize {
        self.ranks.len()
    }

    /// Check if there are no ranks
    pub fn is_empty(&self) -> bool {
        self.ranks.is_empty()
    }

    /// Get rank by level
    pub fn get_by_level(&self, level: u32) -> Option<&RankDefinition> {
        self.ranks.values().find(|r| r.level == level)
    }
}

impl Default for RankDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_level_ordering() {
        assert!(AuthorityLevel::Private < AuthorityLevel::SquadLeader);
        assert!(AuthorityLevel::SquadLeader < AuthorityLevel::Captain);
        assert!(AuthorityLevel::Captain < AuthorityLevel::Commander);
        assert!(AuthorityLevel::Commander < AuthorityLevel::SupremeCommander);
    }

    #[test]
    fn test_authority_level_value() {
        assert_eq!(AuthorityLevel::Private.value(), 0);
        assert_eq!(AuthorityLevel::SquadLeader.value(), 1);
        assert_eq!(AuthorityLevel::Captain.value(), 2);
        assert_eq!(AuthorityLevel::Commander.value(), 3);
        assert_eq!(AuthorityLevel::SupremeCommander.value(), 4);
    }

    #[test]
    fn test_authority_can_command() {
        let captain = AuthorityLevel::Captain;
        let squad_leader = AuthorityLevel::SquadLeader;
        let private = AuthorityLevel::Private;

        assert!(captain.can_command(&squad_leader));
        assert!(captain.can_command(&private));
        assert!(squad_leader.can_command(&private));

        assert!(!private.can_command(&squad_leader));
        assert!(!squad_leader.can_command(&captain));
    }

    #[test]
    fn test_rank_definition_creation() {
        let rank = RankDefinition::new("private", "Private", 0, AuthorityLevel::Private);

        assert_eq!(rank.id, "private");
        assert_eq!(rank.name, "Private");
        assert_eq!(rank.level, 0);
        assert_eq!(rank.authority_level, AuthorityLevel::Private);
        assert_eq!(rank.max_direct_subordinates, 0);
    }

    #[test]
    fn test_rank_definition_default_subordinates() {
        let private = RankDefinition::new("private", "Private", 0, AuthorityLevel::Private);
        let squad_leader =
            RankDefinition::new("sergeant", "Sergeant", 1, AuthorityLevel::SquadLeader);
        let captain = RankDefinition::new("captain", "Captain", 2, AuthorityLevel::Captain);

        assert_eq!(private.max_direct_subordinates, 0);
        assert_eq!(squad_leader.max_direct_subordinates, 5);
        assert_eq!(captain.max_direct_subordinates, 20);
    }

    #[test]
    fn test_rank_definition_with_custom_subordinates() {
        let rank = RankDefinition::new("sergeant", "Sergeant", 1, AuthorityLevel::SquadLeader)
            .with_max_subordinates(10);

        assert_eq!(rank.max_direct_subordinates, 10);
    }

    #[test]
    fn test_rank_can_promote_to() {
        let private = RankDefinition::new("private", "Private", 0, AuthorityLevel::Private);
        let sergeant = RankDefinition::new("sergeant", "Sergeant", 1, AuthorityLevel::SquadLeader);
        let captain = RankDefinition::new("captain", "Captain", 2, AuthorityLevel::Captain);

        assert!(private.can_promote_to(&sergeant));
        assert!(!private.can_promote_to(&captain)); // Skip level
        assert!(sergeant.can_promote_to(&captain));
        assert!(!sergeant.can_promote_to(&private)); // Demotion
    }

    #[test]
    fn test_rank_definitions_add_and_get() {
        let mut defs = RankDefinitions::new();
        let rank = RankDefinition::new("private", "Private", 0, AuthorityLevel::Private);

        defs.add(rank.clone());

        assert_eq!(defs.len(), 1);
        assert_eq!(defs.get(&"private".to_string()), Some(&rank));
    }

    #[test]
    fn test_rank_definitions_get_next_rank() {
        let mut defs = RankDefinitions::new();
        defs.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));
        defs.add(RankDefinition::new(
            "sergeant",
            "Sergeant",
            1,
            AuthorityLevel::SquadLeader,
        ));
        defs.add(RankDefinition::new(
            "captain",
            "Captain",
            2,
            AuthorityLevel::Captain,
        ));

        let next = defs.get_next_rank(&"private".to_string());
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, "sergeant");

        let next = defs.get_next_rank(&"sergeant".to_string());
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, "captain");

        let next = defs.get_next_rank(&"captain".to_string());
        assert!(next.is_none()); // No next rank
    }

    #[test]
    fn test_rank_definitions_get_all_sorted() {
        let mut defs = RankDefinitions::new();
        defs.add(RankDefinition::new(
            "captain",
            "Captain",
            2,
            AuthorityLevel::Captain,
        ));
        defs.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));
        defs.add(RankDefinition::new(
            "sergeant",
            "Sergeant",
            1,
            AuthorityLevel::SquadLeader,
        ));

        let sorted = defs.get_all_sorted();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].id, "private");
        assert_eq!(sorted[1].id, "sergeant");
        assert_eq!(sorted[2].id, "captain");
    }

    #[test]
    fn test_rank_definitions_get_by_level() {
        let mut defs = RankDefinitions::new();
        defs.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));
        defs.add(RankDefinition::new(
            "sergeant",
            "Sergeant",
            1,
            AuthorityLevel::SquadLeader,
        ));

        let rank = defs.get_by_level(0);
        assert!(rank.is_some());
        assert_eq!(rank.unwrap().id, "private");

        let rank = defs.get_by_level(1);
        assert!(rank.is_some());
        assert_eq!(rank.unwrap().id, "sergeant");

        let rank = defs.get_by_level(99);
        assert!(rank.is_none());
    }

    #[test]
    fn test_rank_definitions_is_empty() {
        let mut defs = RankDefinitions::new();
        assert!(defs.is_empty());

        defs.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));
        assert!(!defs.is_empty());
    }

    #[test]
    fn test_rank_serialization() {
        let rank = RankDefinition::new("private", "Private", 0, AuthorityLevel::Private);

        let json = serde_json::to_string(&rank).unwrap();
        let deserialized: RankDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(rank, deserialized);
    }

    #[test]
    fn test_rank_definitions_serialization() {
        let mut defs = RankDefinitions::new();
        defs.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));

        let json = serde_json::to_string(&defs).unwrap();
        let deserialized: RankDefinitions = serde_json::from_str(&json).unwrap();

        assert_eq!(defs.len(), deserialized.len());
        assert_eq!(
            defs.get(&"private".to_string()),
            deserialized.get(&"private".to_string())
        );
    }
}
