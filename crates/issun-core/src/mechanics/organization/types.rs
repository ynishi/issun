//! Type definitions for organization mechanic.

use std::collections::HashMap;

/// Unique identifier for members
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberId(pub String);

impl From<&str> for MemberId {
    fn from(s: &str) -> Self {
        MemberId(s.to_string())
    }
}

impl From<String> for MemberId {
    fn from(s: String) -> Self {
        MemberId(s)
    }
}

// ============================================================================
// Organization Type
// ============================================================================

/// Types of organizational structures
///
/// Each type fundamentally affects how decisions are made, authority is
/// distributed, and members relate to the organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrganizationType {
    /// Hierarchical structure
    /// - Fast decisions (top-down)
    /// - Concentrated power
    /// - Clear chain of command
    /// - Example: Military, Traditional corporations
    Hierarchy,

    /// Democratic structure
    /// - Slow decisions (deliberation)
    /// - Distributed power
    /// - Consensus-driven
    /// - Example: Cooperatives, Republics
    Democracy,

    /// Cult structure
    /// - Instant decisions (charisma)
    /// - Absolute leader authority
    /// - Extreme loyalty requirements
    /// - Example: Borderlands Bandits, Religious cults
    Cult,

    /// Holacratic structure
    /// - Medium decisions (role-based)
    /// - Dynamic authority
    /// - Self-organizing
    /// - Example: Modern tech companies, Agile teams
    Holacracy,

    /// Tribal structure
    /// - Elder-based decisions
    /// - Tradition-focused
    /// - Age/experience hierarchy
    /// - Example: Indigenous communities, Clans
    Tribal,

    /// Corporate structure
    /// - Profit-driven decisions
    /// - Hierarchical but bureaucratic
    /// - Stakeholder interests
    /// - Example: Hyperion, Maliwan (Borderlands megacorps)
    Corporate,

    /// Anarchic structure
    /// - Decentralized decisions
    /// - No formal authority
    /// - Individual autonomy
    /// - Example: Hacker collectives, Loose alliances
    Anarchy,
}

impl Default for OrganizationType {
    fn default() -> Self {
        OrganizationType::Hierarchy
    }
}

// ============================================================================
// Member Archetype
// ============================================================================

/// Member personality archetypes that affect organizational fit
///
/// Members with archetypes that match their organization type have higher
/// loyalty and productivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberArchetype {
    /// Prefers strong leadership and clear hierarchy
    /// - High loyalty in: Hierarchy, Corporate
    /// - Low loyalty in: Anarchy, Holacracy
    Authoritarian,

    /// Prefers equal distribution of power
    /// - High loyalty in: Democracy, Anarchy
    /// - Low loyalty in: Cult, Hierarchy
    Egalitarian,

    /// Highly devoted to charismatic leaders
    /// - High loyalty in: Cult
    /// - Moderate elsewhere
    Devotee,

    /// Values autonomy and self-direction
    /// - High loyalty in: Holacracy, Anarchy
    /// - Low loyalty in: Cult, Hierarchy
    Autonomous,

    /// Pragmatic, adapts to any structure
    /// - Moderate loyalty everywhere
    Pragmatic,

    /// Respects tradition and elders
    /// - High loyalty in: Tribal
    /// - Moderate elsewhere
    Traditionalist,

    /// Driven by profit and advancement
    /// - High loyalty in: Corporate
    /// - Low loyalty in: Tribal, Anarchy
    Ambitious,
}

impl Default for MemberArchetype {
    fn default() -> Self {
        MemberArchetype::Pragmatic
    }
}

// ============================================================================
// Config
// ============================================================================

/// Configuration parameters for organization mechanic
#[derive(Debug, Clone)]
pub struct OrganizationConfig {
    /// Base decision speeds per organization type (multiplier)
    pub base_decision_speeds: HashMap<OrganizationType, f32>,

    /// Consensus thresholds per organization type (0.0-1.0)
    pub consensus_thresholds: HashMap<OrganizationType, f32>,

    /// Authority concentration factor (0.0 = flat, 1.0 = concentrated)
    pub authority_concentration: f32,

    /// Minimum decision speed floor
    pub min_decision_speed: f32,

    /// Maximum decision speed ceiling
    pub max_decision_speed: f32,

    /// Charisma influence factor (for Cult type)
    pub charisma_influence: f32,

    /// Member count scaling factor
    pub member_count_scaling: f32,

    /// Urgency bonus multiplier
    pub urgency_bonus: f32,
}

impl Default for OrganizationConfig {
    fn default() -> Self {
        let mut base_decision_speeds = HashMap::new();
        base_decision_speeds.insert(OrganizationType::Hierarchy, 1.5);
        base_decision_speeds.insert(OrganizationType::Democracy, 0.5);
        base_decision_speeds.insert(OrganizationType::Cult, 2.0);
        base_decision_speeds.insert(OrganizationType::Holacracy, 1.0);
        base_decision_speeds.insert(OrganizationType::Tribal, 0.8);
        base_decision_speeds.insert(OrganizationType::Corporate, 1.2);
        base_decision_speeds.insert(OrganizationType::Anarchy, 0.3);

        let mut consensus_thresholds = HashMap::new();
        consensus_thresholds.insert(OrganizationType::Hierarchy, 0.1);
        consensus_thresholds.insert(OrganizationType::Democracy, 0.51);
        consensus_thresholds.insert(OrganizationType::Cult, 0.0);
        consensus_thresholds.insert(OrganizationType::Holacracy, 0.67);
        consensus_thresholds.insert(OrganizationType::Tribal, 0.75);
        consensus_thresholds.insert(OrganizationType::Corporate, 0.3);
        consensus_thresholds.insert(OrganizationType::Anarchy, 0.9);

        Self {
            base_decision_speeds,
            consensus_thresholds,
            authority_concentration: 0.7,
            min_decision_speed: 0.1,
            max_decision_speed: 3.0,
            charisma_influence: 0.5,
            member_count_scaling: 0.1,
            urgency_bonus: 0.5,
        }
    }
}

// ============================================================================
// Input
// ============================================================================

/// Input snapshot for organization mechanic
#[derive(Debug, Clone)]
pub struct OrganizationInput {
    /// Current organizational type
    pub org_type: OrganizationType,

    /// Number of members in the organization
    pub member_count: usize,

    /// Importance of the current decision (0.0-1.0)
    pub decision_importance: f32,

    /// Urgency of the decision (0.0-1.0)
    pub urgency: f32,

    /// Leader's charisma (for Cult type, 0.0-1.0)
    pub leader_charisma: f32,

    /// Member archetypes (member_id -> archetype)
    pub member_archetypes: Vec<(MemberId, MemberArchetype)>,

    /// Current tick (for event timestamps)
    pub current_tick: u64,
}

impl Default for OrganizationInput {
    fn default() -> Self {
        Self {
            org_type: OrganizationType::default(),
            member_count: 10,
            decision_importance: 0.5,
            urgency: 0.5,
            leader_charisma: 0.5,
            member_archetypes: Vec::new(),
            current_tick: 0,
        }
    }
}

// ============================================================================
// State
// ============================================================================

/// State output from organization mechanic
#[derive(Debug, Clone)]
pub struct OrganizationState {
    /// Calculated decision speed multiplier
    pub decision_speed: f32,

    /// Required consensus percentage (0.0-1.0)
    pub consensus_requirement: f32,

    /// Authority distribution across members (member_id -> authority weight)
    pub authority_distribution: HashMap<MemberId, f32>,

    /// Overall organizational efficiency (0.0-1.0)
    pub efficiency: f32,

    /// Loyalty modifiers per member (member_id -> loyalty modifier)
    pub loyalty_modifiers: HashMap<MemberId, f32>,

    /// Cohesion index (how unified the organization is, 0.0-1.0)
    pub cohesion: f32,

    /// Last update tick
    pub last_update: u64,
}

impl Default for OrganizationState {
    fn default() -> Self {
        Self {
            decision_speed: 1.0,
            consensus_requirement: 0.5,
            authority_distribution: HashMap::new(),
            efficiency: 0.8,
            loyalty_modifiers: HashMap::new(),
            cohesion: 0.8,
            last_update: 0,
        }
    }
}

// ============================================================================
// Events
// ============================================================================

/// Events emitted by organization mechanic
#[derive(Debug, Clone, PartialEq)]
pub enum OrganizationEvent {
    /// Decision-making dynamics calculated
    DecisionDynamicsCalculated {
        /// Decision speed multiplier
        decision_speed: f32,
        /// Required consensus
        consensus_requirement: f32,
    },

    /// Organizational efficiency changed significantly
    EfficiencyChanged {
        /// Old efficiency value
        old_efficiency: f32,
        /// New efficiency value
        new_efficiency: f32,
        /// Reason for change
        reason: EfficiencyChangeReason,
    },

    /// Member loyalty modifier calculated
    LoyaltyModified {
        /// Affected member
        member_id: MemberId,
        /// Loyalty modifier (1.0 = neutral, >1.0 = bonus, <1.0 = penalty)
        modifier: f32,
        /// Archetype-org fit quality
        fit_quality: FitQuality,
    },

    /// Authority distribution rebalanced
    AuthorityRebalanced {
        /// Member with highest authority
        top_authority: Option<MemberId>,
        /// Authority concentration index (0.0-1.0)
        concentration_index: f32,
    },

    /// Cohesion changed
    CohesionChanged {
        /// Old cohesion value
        old_cohesion: f32,
        /// New cohesion value
        new_cohesion: f32,
    },
}

/// Reason for efficiency change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfficiencyChangeReason {
    /// Member count changed
    MemberCountChange,
    /// Archetype distribution changed
    ArchetypeDistributionChange,
    /// Organization type mismatch
    OrganizationTypeMismatch,
    /// High urgency stress
    UrgencyStress,
}

/// Quality of fit between member archetype and organization type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitQuality {
    /// Excellent fit (>1.2 modifier)
    Excellent,
    /// Good fit (1.0-1.2 modifier)
    Good,
    /// Neutral fit (~1.0 modifier)
    Neutral,
    /// Poor fit (0.8-1.0 modifier)
    Poor,
    /// Terrible fit (<0.8 modifier)
    Terrible,
}

impl FitQuality {
    /// Create FitQuality from modifier value
    pub fn from_modifier(modifier: f32) -> Self {
        if modifier >= 1.3 {
            FitQuality::Excellent
        } else if modifier >= 1.1 {
            FitQuality::Good
        } else if modifier >= 0.9 {
            FitQuality::Neutral
        } else if modifier >= 0.7 {
            FitQuality::Poor
        } else {
            FitQuality::Terrible
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_config_default() {
        let config = OrganizationConfig::default();
        assert_eq!(
            config.base_decision_speeds.get(&OrganizationType::Cult),
            Some(&2.0)
        );
        assert_eq!(
            config.consensus_thresholds.get(&OrganizationType::Democracy),
            Some(&0.51)
        );
    }

    #[test]
    fn test_organization_state_default() {
        let state = OrganizationState::default();
        assert_eq!(state.decision_speed, 1.0);
        assert_eq!(state.efficiency, 0.8);
    }

    #[test]
    fn test_fit_quality_from_modifier() {
        assert_eq!(FitQuality::from_modifier(1.5), FitQuality::Excellent);
        assert_eq!(FitQuality::from_modifier(1.15), FitQuality::Good);
        assert_eq!(FitQuality::from_modifier(1.0), FitQuality::Neutral);
        assert_eq!(FitQuality::from_modifier(0.85), FitQuality::Poor);
        assert_eq!(FitQuality::from_modifier(0.5), FitQuality::Terrible);
    }

    #[test]
    fn test_member_id_from_string() {
        let id: MemberId = "test_member".into();
        assert_eq!(id.0, "test_member");
    }
}
