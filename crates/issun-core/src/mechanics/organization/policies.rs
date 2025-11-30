//! Policy traits for organization mechanic.

use super::types::{
    MemberArchetype, MemberId, OrganizationConfig, OrganizationInput, OrganizationState,
    OrganizationType,
};
use std::collections::HashMap;

/// Policy for organizational behavior and dynamics
///
/// This trait defines how organizational characteristics are calculated
/// based on the organization type, member composition, and context.
/// Different implementations can model different organizational theories
/// (bureaucratic, agile, charismatic, etc.).
pub trait OrganizationPolicy {
    /// Calculate decision-making speed
    ///
    /// # Arguments
    ///
    /// * `config` - Organization configuration
    /// * `input` - Current organization input (type, members, urgency)
    ///
    /// # Returns
    ///
    /// Decision speed multiplier (>1.0 = faster, <1.0 = slower)
    fn calculate_decision_speed(config: &OrganizationConfig, input: &OrganizationInput) -> f32;

    /// Calculate consensus requirement for decisions
    ///
    /// # Arguments
    ///
    /// * `config` - Organization configuration
    /// * `input` - Current organization input
    ///
    /// # Returns
    ///
    /// Required agreement percentage (0.0-1.0)
    fn calculate_consensus_requirement(
        config: &OrganizationConfig,
        input: &OrganizationInput,
    ) -> f32;

    /// Determine authority distribution across members
    ///
    /// # Arguments
    ///
    /// * `config` - Organization configuration
    /// * `input` - Current organization input
    ///
    /// # Returns
    ///
    /// HashMap of member_id -> authority weight (weights sum to 1.0)
    fn determine_authority_distribution(
        config: &OrganizationConfig,
        input: &OrganizationInput,
    ) -> HashMap<MemberId, f32>;

    /// Calculate loyalty modifier for a member based on archetype fit
    ///
    /// # Arguments
    ///
    /// * `org_type` - Organization type
    /// * `archetype` - Member's archetype
    ///
    /// # Returns
    ///
    /// Loyalty modifier (1.0 = neutral, >1.0 = bonus, <1.0 = penalty)
    fn calculate_loyalty_modifier(org_type: &OrganizationType, archetype: &MemberArchetype) -> f32;

    /// Calculate organizational efficiency
    ///
    /// # Arguments
    ///
    /// * `config` - Organization configuration
    /// * `state` - Current organization state
    /// * `input` - Current organization input
    ///
    /// # Returns
    ///
    /// Efficiency value (0.0-1.0)
    fn calculate_efficiency(
        config: &OrganizationConfig,
        state: &OrganizationState,
        input: &OrganizationInput,
    ) -> f32;

    /// Calculate organizational cohesion
    ///
    /// # Arguments
    ///
    /// * `loyalty_modifiers` - Map of member loyalty modifiers
    ///
    /// # Returns
    ///
    /// Cohesion value (0.0-1.0)
    fn calculate_cohesion(loyalty_modifiers: &HashMap<MemberId, f32>) -> f32;
}
