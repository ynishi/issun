//! Core data types for SocialPlugin
//!
//! Implements social network analysis, influence graphs, and political dynamics
//! for simulation of informal organizational power structures.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Unique identifier for a member
pub type MemberId = String;

/// Unique identifier for a faction/organization
pub type FactionId = String;

/// Types of social relationships between members
///
/// Unlike formal hierarchy (commands), these represent informal connections
/// that constitute the real power structure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    /// Trust relationship (bidirectional, 0.0-1.0)
    Trust { strength: f32 },

    /// Debt/favor (directional, -1.0 ~ 1.0)
    /// Positive: they owe me, Negative: I owe them
    Debt { amount: f32 },

    /// Shared secret (mutual dependency)
    SharedSecret { secret_id: String, sensitivity: f32 },

    /// Faction membership (belong to same faction)
    FactionMembership { faction_id: FactionId },

    /// Hostility relationship (bidirectional, 0.0-1.0)
    Hostility { intensity: f32 },

    /// Custom relationship type
    Custom(String),
}

impl RelationType {
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            RelationType::Trust { strength } => format!("Trust ({})", strength),
            RelationType::Debt { amount } if *amount > 0.0 => {
                format!("They owe me ({})", amount)
            }
            RelationType::Debt { amount } => format!("I owe them ({})", -amount),
            RelationType::SharedSecret { secret_id, .. } => {
                format!("Shared secret: {}", secret_id)
            }
            RelationType::FactionMembership { faction_id } => {
                format!("Faction: {}", faction_id)
            }
            RelationType::Hostility { intensity } => format!("Hostility ({})", intensity),
            RelationType::Custom(name) => name.clone(),
        }
    }

    /// Check if this relation creates obligation
    pub fn creates_obligation(&self) -> bool {
        matches!(
            self,
            RelationType::Debt { .. } | RelationType::SharedSecret { .. }
        )
    }
}

/// Network centrality metrics - measures of influence in social network
///
/// Based on graph theory, these metrics quantify different aspects
/// of power and influence within an organization.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CentralityMetrics {
    /// Degree centrality: number of direct connections
    /// "How well-connected"
    pub degree: f32,

    /// Betweenness centrality: frequency of being on shortest paths
    /// "How much of an information broker"
    pub betweenness: f32,

    /// Closeness centrality: average distance to all nodes
    /// "How fast information spreads from this person"
    pub closeness: f32,

    /// Eigenvector centrality: connections to influential people
    /// "How close to power centers"
    pub eigenvector: f32,

    /// Overall influence score (weighted average)
    pub overall_influence: f32,
}

impl Default for CentralityMetrics {
    fn default() -> Self {
        Self {
            degree: 0.0,
            betweenness: 0.0,
            closeness: 0.0,
            eigenvector: 0.0,
            overall_influence: 0.0,
        }
    }
}

impl CentralityMetrics {
    /// Calculate overall influence from components
    pub fn calculate_overall(
        &mut self,
        degree_weight: f32,
        betweenness_weight: f32,
        closeness_weight: f32,
        eigenvector_weight: f32,
    ) {
        self.overall_influence = self.degree * degree_weight
            + self.betweenness * betweenness_weight
            + self.closeness * closeness_weight
            + self.eigenvector * eigenvector_weight;
    }

    /// Check if this member is a "shadow leader" (KingMaker)
    pub fn is_shadow_leader(&self, threshold: f32) -> bool {
        self.overall_influence > threshold
    }
}

/// Social capital - the "political currency" a member holds
///
/// Unlike formal authority, this represents informal power
/// accumulated through relationships and secrets.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SocialCapital {
    /// Reputation/prestige (0.0-1.0)
    pub reputation: f32,

    /// Total favors others owe to this member
    pub total_favors_owed_to_me: f32,

    /// Total favors this member owes to others
    pub total_favors_i_owe: f32,

    /// Number of secrets this member knows
    pub secrets_held: u32,

    /// Network centrality scores
    pub centrality_scores: CentralityMetrics,
}

impl Default for SocialCapital {
    fn default() -> Self {
        Self {
            reputation: 0.5,
            total_favors_owed_to_me: 0.0,
            total_favors_i_owe: 0.0,
            secrets_held: 0,
            centrality_scores: CentralityMetrics::default(),
        }
    }
}

impl SocialCapital {
    /// Get net favor balance (positive = creditor, negative = debtor)
    pub fn net_favor_balance(&self) -> f32 {
        self.total_favors_owed_to_me - self.total_favors_i_owe
    }

    /// Check if member can afford a political action
    pub fn can_afford(&self, cost: f32) -> bool {
        self.net_favor_balance() >= cost
    }
}

/// Political faction - informal coalition based on shared interests
///
/// Unlike formal organizations (Hierarchy), factions are fluid
/// and can form, split, or merge based on changing circumstances.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,

    /// Faction members
    pub members: HashSet<MemberId>,

    /// Faction leader (member with highest influence)
    pub leader: Option<MemberId>,

    /// Faction agenda/goals
    pub agenda: Vec<String>,

    /// Faction cohesion (0.0-1.0) - likelihood to stay together
    pub cohesion: f32,

    /// Relations with other factions (-1.0 ~ 1.0)
    pub inter_faction_relations: HashMap<FactionId, f32>,
}

impl Faction {
    /// Create a new faction
    pub fn new(id: FactionId, name: String) -> Self {
        Self {
            id,
            name,
            members: HashSet::new(),
            leader: None,
            agenda: Vec::new(),
            cohesion: 1.0,
            inter_faction_relations: HashMap::new(),
        }
    }

    /// Add member to faction
    pub fn add_member(&mut self, member_id: MemberId) {
        self.members.insert(member_id);
    }

    /// Remove member from faction
    pub fn remove_member(&mut self, member_id: &MemberId) -> bool {
        self.members.remove(member_id)
    }

    /// Check if faction is allied with another
    pub fn is_allied_with(&self, other_faction_id: &FactionId) -> bool {
        self.inter_faction_relations
            .get(other_faction_id)
            .map(|&relation| relation > 0.5)
            .unwrap_or(false)
    }

    /// Check if faction is hostile to another
    pub fn is_hostile_to(&self, other_faction_id: &FactionId) -> bool {
        self.inter_faction_relations
            .get(other_faction_id)
            .map(|&relation| relation < -0.5)
            .unwrap_or(false)
    }
}

/// Political actions members can perform
///
/// These represent informal power moves outside official channels.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PoliticalAction {
    /// Lobbying - gain support before formal decision
    Lobbying {
        target: MemberId,
        proposal: String,
        cost: f32,
    },

    /// Grant favor - help someone to create debt
    GrantFavor { target: MemberId, favor_value: f32 },

    /// Call in favor - cash in accumulated debt
    CallInFavor { target: MemberId, request: String },

    /// Share secret - create mutual dependency
    ShareSecret {
        target: MemberId,
        secret_id: String,
        sensitivity: f32,
    },

    /// Spread gossip - manipulate reputation
    SpreadGossip {
        about: MemberId,
        content: String,
        is_positive: bool,
    },

    /// Form coalition - create or expand faction
    FormCoalition {
        members: Vec<MemberId>,
        agenda: String,
    },

    /// Defect - leave faction for another
    Defect {
        from_faction: FactionId,
        to_faction: Option<FactionId>,
    },
}

impl PoliticalAction {
    /// Get the target member if action has one
    pub fn target(&self) -> Option<&MemberId> {
        match self {
            PoliticalAction::Lobbying { target, .. }
            | PoliticalAction::GrantFavor { target, .. }
            | PoliticalAction::CallInFavor { target, .. }
            | PoliticalAction::ShareSecret { target, .. }
            | PoliticalAction::SpreadGossip { about: target, .. } => Some(target),
            _ => None,
        }
    }

    /// Get the cost of this action
    pub fn cost(&self) -> f32 {
        match self {
            PoliticalAction::Lobbying { cost, .. } => *cost,
            PoliticalAction::GrantFavor { favor_value, .. } => *favor_value,
            PoliticalAction::ShareSecret { sensitivity, .. } => *sensitivity * 0.5,
            _ => 0.0,
        }
    }
}

/// Errors that can occur in social network operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SocialError {
    /// Member not found in network
    #[error("Member not found: {0}")]
    MemberNotFound(MemberId),

    /// Faction not found
    #[error("Faction not found: {0}")]
    FactionNotFound(FactionId),

    /// Insufficient social capital for action
    #[error("Insufficient social capital: need {required}, have {available}")]
    InsufficientCapital { required: f32, available: f32 },

    /// Invalid relationship
    #[error("Invalid relationship between {from} and {to}")]
    InvalidRelationship { from: MemberId, to: MemberId },

    /// Political action failed
    #[error("Political action failed: {0}")]
    ActionFailed(String),

    /// Network analysis error
    #[error("Network analysis error: {0}")]
    AnalysisError(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_type_description() {
        let trust = RelationType::Trust { strength: 0.8 };
        assert!(trust.description().contains("Trust"));

        let debt = RelationType::Debt { amount: 1.0 };
        assert!(debt.description().contains("They owe me"));
    }

    #[test]
    fn test_relation_creates_obligation() {
        assert!(RelationType::Debt { amount: 1.0 }.creates_obligation());
        assert!(RelationType::SharedSecret {
            secret_id: "test".to_string(),
            sensitivity: 0.5
        }
        .creates_obligation());
        assert!(!RelationType::Trust { strength: 0.8 }.creates_obligation());
    }

    #[test]
    fn test_centrality_metrics_default() {
        let metrics = CentralityMetrics::default();
        assert_eq!(metrics.degree, 0.0);
        assert_eq!(metrics.overall_influence, 0.0);
    }

    #[test]
    fn test_centrality_calculate_overall() {
        let mut metrics = CentralityMetrics {
            degree: 0.5,
            betweenness: 0.8,
            closeness: 0.3,
            eigenvector: 0.6,
            overall_influence: 0.0,
        };

        metrics.calculate_overall(0.3, 0.3, 0.2, 0.2);

        // 0.5*0.3 + 0.8*0.3 + 0.3*0.2 + 0.6*0.2 = 0.15 + 0.24 + 0.06 + 0.12 = 0.57
        assert!((metrics.overall_influence - 0.57).abs() < 0.01);
    }

    #[test]
    fn test_centrality_is_shadow_leader() {
        let metrics = CentralityMetrics {
            overall_influence: 0.85,
            ..Default::default()
        };

        assert!(metrics.is_shadow_leader(0.75));
        assert!(!metrics.is_shadow_leader(0.90));
    }

    #[test]
    fn test_social_capital_default() {
        let capital = SocialCapital::default();
        assert_eq!(capital.reputation, 0.5);
        assert_eq!(capital.net_favor_balance(), 0.0);
    }

    #[test]
    fn test_social_capital_net_favor_balance() {
        let capital = SocialCapital {
            total_favors_owed_to_me: 10.0,
            total_favors_i_owe: 3.0,
            ..Default::default()
        };

        assert_eq!(capital.net_favor_balance(), 7.0);
        assert!(capital.can_afford(5.0));
        assert!(!capital.can_afford(10.0));
    }

    #[test]
    fn test_faction_new() {
        let faction = Faction::new("faction_1".to_string(), "Test Faction".to_string());
        assert_eq!(faction.id, "faction_1");
        assert_eq!(faction.name, "Test Faction");
        assert_eq!(faction.cohesion, 1.0);
        assert!(faction.members.is_empty());
    }

    #[test]
    fn test_faction_add_remove_member() {
        let mut faction = Faction::new("f1".to_string(), "Test".to_string());

        faction.add_member("member1".to_string());
        assert!(faction.members.contains("member1"));

        let removed = faction.remove_member(&"member1".to_string());
        assert!(removed);
        assert!(!faction.members.contains("member1"));
    }

    #[test]
    fn test_faction_relations() {
        let mut faction = Faction::new("f1".to_string(), "Test".to_string());

        faction
            .inter_faction_relations
            .insert("ally".to_string(), 0.8);
        faction
            .inter_faction_relations
            .insert("enemy".to_string(), -0.8);

        assert!(faction.is_allied_with(&"ally".to_string()));
        assert!(faction.is_hostile_to(&"enemy".to_string()));
        assert!(!faction.is_allied_with(&"unknown".to_string()));
    }

    #[test]
    fn test_political_action_target() {
        let action = PoliticalAction::Lobbying {
            target: "member1".to_string(),
            proposal: "Test".to_string(),
            cost: 1.0,
        };

        assert_eq!(action.target(), Some(&"member1".to_string()));
        assert_eq!(action.cost(), 1.0);
    }

    #[test]
    fn test_political_action_cost() {
        let grant = PoliticalAction::GrantFavor {
            target: "m1".to_string(),
            favor_value: 2.0,
        };
        assert_eq!(grant.cost(), 2.0);

        let secret = PoliticalAction::ShareSecret {
            target: "m2".to_string(),
            secret_id: "s1".to_string(),
            sensitivity: 1.0,
        };
        assert_eq!(secret.cost(), 0.5); // sensitivity * 0.5
    }

    #[test]
    fn test_social_error_display() {
        let err = SocialError::MemberNotFound("member1".to_string());
        assert!(err.to_string().contains("member1"));

        let err = SocialError::InsufficientCapital {
            required: 10.0,
            available: 5.0,
        };
        assert!(err.to_string().contains("10"));
        assert!(err.to_string().contains("5"));
    }
}
