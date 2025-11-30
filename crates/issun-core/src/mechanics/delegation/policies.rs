//! Policy traits for delegation mechanic.

use super::types::{
    DelegateStats, DelegateTrait, DelegationConfig, DelegationInput, DelegatorStats, DirectiveType,
};

/// Policy for delegation behavior and compliance calculation
///
/// This trait defines how delegation dynamics are calculated based on
/// the delegator, delegate, directive type, and context.
/// Different implementations can model different organizational cultures
/// (military, cooperative, corporate, etc.).
pub trait DelegationPolicy {
    /// Calculate compliance probability
    ///
    /// # Arguments
    ///
    /// * `config` - Delegation configuration
    /// * `input` - Current delegation input
    ///
    /// # Returns
    ///
    /// Compliance value (-1.0 to 1.0)
    /// - Positive: will comply (higher = more faithful)
    /// - Zero: indifferent
    /// - Negative: will defy (lower = more active resistance)
    fn calculate_compliance(config: &DelegationConfig, input: &DelegationInput) -> f32;

    /// Calculate interpretation level
    ///
    /// How much creative freedom the delegate will take in executing the directive.
    ///
    /// # Returns
    ///
    /// Interpretation value (0.0 = literal, 1.0 = highly creative)
    fn calculate_interpretation(config: &DelegationConfig, input: &DelegationInput) -> f32;

    /// Calculate priority assigned by delegate
    ///
    /// How important the delegate considers this directive relative to other work.
    ///
    /// # Returns
    ///
    /// Priority value (0.0-1.0)
    fn calculate_priority(config: &DelegationConfig, input: &DelegationInput) -> f32;

    /// Calculate feedback probability
    ///
    /// Likelihood that the delegate will report progress/completion.
    ///
    /// # Returns
    ///
    /// Feedback probability (0.0-1.0)
    fn calculate_feedback_probability(config: &DelegationConfig, input: &DelegationInput) -> f32;

    /// Calculate expected execution quality
    ///
    /// Based on delegate skills, workload, and compliance.
    ///
    /// # Returns
    ///
    /// Expected quality (0.0-1.0)
    fn calculate_expected_quality(config: &DelegationConfig, input: &DelegationInput) -> f32;

    /// Calculate propagation delay
    ///
    /// Time for directive to be acknowledged and acted upon.
    ///
    /// # Returns
    ///
    /// Delay in ticks
    fn calculate_propagation_delay(config: &DelegationConfig, input: &DelegationInput) -> f32;

    /// Calculate trait modifier for compliance
    ///
    /// How the delegate's personality affects compliance.
    ///
    /// # Returns
    ///
    /// Modifier value (multiplier, 1.0 = neutral)
    fn calculate_trait_modifier(
        delegate_trait: &DelegateTrait,
        directive_type: &DirectiveType,
        delegator: &DelegatorStats,
        delegate: &DelegateStats,
    ) -> f32;
}
