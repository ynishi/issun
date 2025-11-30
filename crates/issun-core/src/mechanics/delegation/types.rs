//! Type definitions for delegation mechanic.

use std::collections::HashMap;

/// Unique identifier for entities (delegator or delegate)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub String);

impl From<&str> for EntityId {
    fn from(s: &str) -> Self {
        EntityId(s.to_string())
    }
}

impl From<String> for EntityId {
    fn from(s: String) -> Self {
        EntityId(s)
    }
}

/// Unique identifier for directives
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveId(pub String);

impl From<&str> for DirectiveId {
    fn from(s: &str) -> Self {
        DirectiveId(s.to_string())
    }
}

impl From<String> for DirectiveId {
    fn from(s: String) -> Self {
        DirectiveId(s)
    }
}

// ============================================================================
// Directive Type
// ============================================================================

/// Types of directives that can be delegated
///
/// Each type has different characteristics for compliance calculation
/// and execution behavior.
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveType {
    /// Direct command from superior (military-style)
    /// - High authority weight
    /// - Low interpretation allowed
    Command {
        /// Target of the command (location, entity, etc.)
        target: String,
        /// Specific action to perform
        action: String,
    },

    /// Request from peer or ally (cooperative)
    /// - Medium authority weight
    /// - Relationship affects compliance
    Request {
        /// What is being requested
        description: String,
        /// Offered compensation or reciprocity
        compensation: Option<f32>,
    },

    /// Task assignment (work delegation)
    /// - Based on role/capability
    /// - Skill match affects execution quality
    Task {
        /// Task identifier
        task_id: String,
        /// Required skill or capability
        required_skill: Option<String>,
        /// Deadline in ticks (None = no deadline)
        deadline: Option<u64>,
    },

    /// Suggestion or advice (low pressure)
    /// - Low authority weight
    /// - High interpretation allowed
    Suggestion {
        /// The suggestion content
        content: String,
    },

    /// Standing order (persistent directive)
    /// - Applies until revoked
    /// - Lower per-instance compliance pressure
    StandingOrder {
        /// Order identifier
        order_id: String,
        /// Conditions under which to act
        trigger_condition: String,
    },
}

impl DirectiveType {
    /// Get the base authority weight for this directive type
    pub fn base_authority_weight(&self) -> f32 {
        match self {
            DirectiveType::Command { .. } => 1.0,
            DirectiveType::Request { .. } => 0.6,
            DirectiveType::Task { .. } => 0.8,
            DirectiveType::Suggestion { .. } => 0.3,
            DirectiveType::StandingOrder { .. } => 0.7,
        }
    }

    /// Get the base interpretation freedom for this directive type
    pub fn base_interpretation_freedom(&self) -> f32 {
        match self {
            DirectiveType::Command { .. } => 0.1,
            DirectiveType::Request { .. } => 0.5,
            DirectiveType::Task { .. } => 0.4,
            DirectiveType::Suggestion { .. } => 0.9,
            DirectiveType::StandingOrder { .. } => 0.6,
        }
    }
}

// ============================================================================
// Delegate Traits
// ============================================================================

/// Personality traits that affect how a delegate handles directives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelegateTrait {
    /// Follows orders precisely, little interpretation
    Obedient,

    /// Questions orders, may refuse unreasonable ones
    Independent,

    /// Enthusiastic execution, may over-deliver
    Eager,

    /// Minimal effort, does bare minimum
    Reluctant,

    /// Loyal to delegator, high compliance
    Loyal,

    /// Opportunistic, compliance depends on self-interest
    Opportunist,

    /// Creative interpretation, may improve on orders
    Innovative,

    /// By-the-book execution, no deviation
    Rigid,
}

impl Default for DelegateTrait {
    fn default() -> Self {
        DelegateTrait::Obedient
    }
}

// ============================================================================
// Config
// ============================================================================

/// Configuration parameters for delegation mechanic
#[derive(Debug, Clone)]
pub struct DelegationConfig {
    /// Base compliance rate before modifiers
    pub base_compliance: f32,

    /// Weight of loyalty in compliance calculation
    pub loyalty_weight: f32,

    /// Weight of relationship in compliance calculation
    pub relationship_weight: f32,

    /// Weight of authority in compliance calculation
    pub authority_weight: f32,

    /// Weight of morale in compliance calculation
    pub morale_weight: f32,

    /// Propagation delay per hierarchy level (in ticks)
    pub propagation_delay_per_level: f32,

    /// Threshold below which directive is ignored
    pub ignore_threshold: f32,

    /// Threshold below which directive triggers defiance
    pub defiance_threshold: f32,

    /// Feedback probability base rate
    pub base_feedback_rate: f32,
}

impl Default for DelegationConfig {
    fn default() -> Self {
        Self {
            base_compliance: 0.7,
            loyalty_weight: 0.3,
            relationship_weight: 0.2,
            authority_weight: 0.25,
            morale_weight: 0.15,
            propagation_delay_per_level: 1.0,
            ignore_threshold: 0.2,
            defiance_threshold: -0.2,
            base_feedback_rate: 0.5,
        }
    }
}

// ============================================================================
// Input
// ============================================================================

/// A directive being delegated
#[derive(Debug, Clone)]
pub struct Directive {
    /// Unique identifier for this directive
    pub id: DirectiveId,

    /// Type of directive
    pub directive_type: DirectiveType,

    /// Urgency level (0.0-1.0, affects priority)
    pub urgency: f32,

    /// Importance level (0.0-1.0, affects compliance pressure)
    pub importance: f32,

    /// Tick when directive was issued
    pub issued_at: u64,
}

/// Statistics about the delegator
#[derive(Debug, Clone)]
pub struct DelegatorStats {
    /// Entity ID of the delegator
    pub entity_id: EntityId,

    /// Authority level (0.0-1.0)
    pub authority: f32,

    /// Charisma (affects persuasion)
    pub charisma: f32,

    /// Hierarchy rank (0 = top)
    pub hierarchy_rank: usize,

    /// Reputation with the delegate (can be negative)
    pub reputation: f32,
}

/// Statistics about the delegate (the one receiving the directive)
#[derive(Debug, Clone)]
pub struct DelegateStats {
    /// Entity ID of the delegate
    pub entity_id: EntityId,

    /// Current loyalty to delegator (-1.0 to 1.0)
    pub loyalty: f32,

    /// Current morale (0.0-1.0)
    pub morale: f32,

    /// Relationship quality with delegator (-1.0 to 1.0)
    pub relationship: f32,

    /// Hierarchy rank (0 = top)
    pub hierarchy_rank: usize,

    /// Personality trait
    pub personality: DelegateTrait,

    /// Current workload (0.0-1.0, affects capacity)
    pub workload: f32,

    /// Skill level for task execution (0.0-1.0)
    pub skill_level: f32,
}

/// Input snapshot for delegation mechanic
#[derive(Debug, Clone)]
pub struct DelegationInput {
    /// The directive being delegated
    pub directive: Directive,

    /// Delegator statistics
    pub delegator: DelegatorStats,

    /// Delegate statistics
    pub delegate: DelegateStats,

    /// Current tick
    pub current_tick: u64,
}

// ============================================================================
// State
// ============================================================================

/// Response type from the delegate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    /// Full compliance, will execute as directed
    Accept,

    /// Partial compliance, will execute with modifications
    AcceptWithReservation,

    /// Delayed response, needs time to decide
    Defer,

    /// Non-compliance, ignores directive
    Ignore,

    /// Active defiance, may act contrary
    Defy,
}

impl ResponseType {
    /// Create from compliance value
    pub fn from_compliance(compliance: f32, config: &DelegationConfig) -> Self {
        if compliance >= 0.8 {
            ResponseType::Accept
        } else if compliance >= 0.5 {
            ResponseType::AcceptWithReservation
        } else if compliance >= config.ignore_threshold {
            ResponseType::Defer
        } else if compliance >= config.defiance_threshold {
            ResponseType::Ignore
        } else {
            ResponseType::Defy
        }
    }
}

/// State output from delegation mechanic
#[derive(Debug, Clone)]
pub struct DelegationState {
    /// Calculated compliance probability (-1.0 to 1.0)
    /// Negative values indicate defiance
    pub compliance: f32,

    /// How the delegate interprets the directive (0.0 = literal, 1.0 = creative)
    pub interpretation: f32,

    /// Priority assigned by delegate (0.0-1.0)
    pub priority: f32,

    /// Probability of providing feedback (0.0-1.0)
    pub feedback_probability: f32,

    /// Expected execution quality (0.0-1.0)
    pub expected_quality: f32,

    /// Propagation delay in ticks
    pub propagation_delay: f32,

    /// Response type
    pub response: ResponseType,

    /// Active directives being tracked
    pub active_directives: HashMap<DirectiveId, DirectiveStatus>,

    /// Last update tick
    pub last_update: u64,
}

impl Default for DelegationState {
    fn default() -> Self {
        Self {
            compliance: 0.7,
            interpretation: 0.3,
            priority: 0.5,
            feedback_probability: 0.5,
            expected_quality: 0.7,
            propagation_delay: 0.0,
            response: ResponseType::Accept,
            active_directives: HashMap::new(),
            last_update: 0,
        }
    }
}

/// Status of a directive being executed
#[derive(Debug, Clone)]
pub struct DirectiveStatus {
    /// Current execution progress (0.0-1.0)
    pub progress: f32,

    /// Current status
    pub status: ExecutionStatus,

    /// Interpretation applied
    pub interpretation_applied: f32,

    /// Quality of execution so far
    pub current_quality: f32,
}

/// Execution status of a directive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Waiting to start
    Pending,

    /// Currently being executed
    InProgress,

    /// Successfully completed
    Completed,

    /// Failed to complete
    Failed,

    /// Deliberately ignored
    Ignored,

    /// Actively defied
    Defied,
}

// ============================================================================
// Events
// ============================================================================

/// Events emitted by delegation mechanic
#[derive(Debug, Clone, PartialEq)]
pub enum DelegationEvent {
    /// Directive received and processed
    DirectiveReceived {
        /// Directive ID
        directive_id: DirectiveId,
        /// Delegator
        delegator: EntityId,
        /// Delegate
        delegate: EntityId,
        /// Response type
        response: ResponseType,
        /// Calculated compliance
        compliance: f32,
    },

    /// Directive accepted (full or partial)
    DirectiveAccepted {
        /// Directive ID
        directive_id: DirectiveId,
        /// Priority assigned
        priority: f32,
        /// Interpretation level
        interpretation: f32,
    },

    /// Directive ignored
    DirectiveIgnored {
        /// Directive ID
        directive_id: DirectiveId,
        /// Reason for ignoring
        reason: IgnoreReason,
    },

    /// Directive defied (active resistance)
    DirectiveDefied {
        /// Directive ID
        directive_id: DirectiveId,
        /// Delegate who defied
        delegate: EntityId,
        /// Severity of defiance (-1.0 to 0.0)
        severity: f32,
    },

    /// Compliance changed significantly
    ComplianceChanged {
        /// Delegate
        delegate: EntityId,
        /// Old compliance
        old_compliance: f32,
        /// New compliance
        new_compliance: f32,
        /// Reason for change
        reason: ComplianceChangeReason,
    },

    /// Feedback provided by delegate
    FeedbackProvided {
        /// Directive ID
        directive_id: DirectiveId,
        /// Progress reported
        progress: f32,
        /// Quality reported
        quality: f32,
    },
}

/// Reason for ignoring a directive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreReason {
    /// Low authority of delegator
    InsufficientAuthority,
    /// Poor relationship
    PoorRelationship,
    /// Low morale
    LowMorale,
    /// Overloaded with work
    CapacityFull,
    /// Directive deemed unimportant
    LowPriority,
}

/// Reason for compliance change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceChangeReason {
    /// Loyalty changed
    LoyaltyChange,
    /// Relationship changed
    RelationshipChange,
    /// Authority perception changed
    AuthorityChange,
    /// Morale changed
    MoraleChange,
    /// Workload changed
    WorkloadChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_config_default() {
        let config = DelegationConfig::default();
        assert_eq!(config.base_compliance, 0.7);
        assert!(config.loyalty_weight > 0.0);
    }

    #[test]
    fn test_delegation_state_default() {
        let state = DelegationState::default();
        assert_eq!(state.compliance, 0.7);
        assert_eq!(state.response, ResponseType::Accept);
    }

    #[test]
    fn test_response_type_from_compliance() {
        let config = DelegationConfig::default();

        assert_eq!(
            ResponseType::from_compliance(0.9, &config),
            ResponseType::Accept
        );
        assert_eq!(
            ResponseType::from_compliance(0.6, &config),
            ResponseType::AcceptWithReservation
        );
        assert_eq!(
            ResponseType::from_compliance(0.3, &config),
            ResponseType::Defer
        );
        assert_eq!(
            ResponseType::from_compliance(0.1, &config),
            ResponseType::Ignore
        );
        assert_eq!(
            ResponseType::from_compliance(-0.5, &config),
            ResponseType::Defy
        );
    }

    #[test]
    fn test_directive_type_weights() {
        let cmd = DirectiveType::Command {
            target: "location".into(),
            action: "move".into(),
        };
        assert_eq!(cmd.base_authority_weight(), 1.0);
        assert_eq!(cmd.base_interpretation_freedom(), 0.1);

        let suggestion = DirectiveType::Suggestion {
            content: "consider this".into(),
        };
        assert_eq!(suggestion.base_authority_weight(), 0.3);
        assert_eq!(suggestion.base_interpretation_freedom(), 0.9);
    }

    #[test]
    fn test_entity_id_from_string() {
        let id: EntityId = "test_entity".into();
        assert_eq!(id.0, "test_entity");
    }
}
