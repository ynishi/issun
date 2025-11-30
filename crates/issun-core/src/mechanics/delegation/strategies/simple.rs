//! Simple delegation policy implementation.

use crate::mechanics::delegation::policies::DelegationPolicy;
use crate::mechanics::delegation::types::{
    DelegateStats, DelegateTrait, DelegationConfig, DelegationInput, DelegatorStats, DirectiveType,
};

/// Simple delegation policy with intuitive calculations
///
/// This policy implements straightforward delegation dynamics:
/// - **Compliance**: Weighted sum of loyalty, relationship, authority, morale
/// - **Interpretation**: Based on directive type and personality
/// - **Priority**: Based on urgency, importance, and relationship
/// - **Quality**: Based on skill, workload, and compliance
///
/// # Example Scenarios
///
/// ```ignore
/// // Loyal soldier receiving command from respected general
/// // - High loyalty (0.9), good relationship (0.7), high authority (0.9)
/// // - Result: Very high compliance (~0.95), low interpretation
///
/// // Independent ally receiving request
/// // - Medium loyalty (0.5), good relationship (0.8), medium authority (0.5)
/// // - Result: Moderate compliance (~0.65), higher interpretation
///
/// // Disgruntled subordinate receiving task
/// // - Low loyalty (0.2), poor relationship (-0.3), high authority (0.8)
/// // - Result: Low compliance (~0.35), may ignore or delay
/// ```
pub struct SimpleDelegationPolicy;

impl DelegationPolicy for SimpleDelegationPolicy {
    fn calculate_compliance(config: &DelegationConfig, input: &DelegationInput) -> f32 {
        let directive = &input.directive;
        let delegator = &input.delegator;
        let delegate = &input.delegate;

        // Start with neutral base
        let mut compliance = 0.0;

        // Loyalty factor (-1.0 to 1.0) - strongest influence
        // Negative loyalty strongly reduces compliance
        compliance += delegate.loyalty * config.loyalty_weight * 2.0;

        // Relationship factor (-1.0 to 1.0)
        // Poor relationship hurts compliance significantly
        compliance += delegate.relationship * config.relationship_weight * 1.5;

        // Authority factor (0.0 to 1.0)
        // Low authority = weak compliance pressure
        let authority_effect = delegator.authority * config.authority_weight;
        compliance += authority_effect;

        // Rank difference penalty (subordinate ignoring superior is harder)
        if delegate.hierarchy_rank > delegator.hierarchy_rank {
            let rank_bonus = ((delegate.hierarchy_rank - delegator.hierarchy_rank) as f32).min(3.0) * 0.1;
            compliance += rank_bonus;
        }

        // Morale factor (0.0 to 1.0 centered at 0.5)
        let morale_factor = (delegate.morale - 0.5) * config.morale_weight;
        compliance += morale_factor;

        // Charisma bonus (0.0 to 0.2)
        compliance += delegator.charisma * 0.15;

        // Reputation bonus/penalty (-0.1 to 0.1)
        compliance += delegator.reputation * 0.1;

        // Directive type weight (Commands are harder to refuse than suggestions)
        let type_weight = directive.directive_type.base_authority_weight();
        // Scale compliance by type weight, but preserve sign for defiance
        if compliance >= 0.0 {
            compliance *= type_weight;
        } else {
            // Negative compliance (defiance) - type weight reduces magnitude
            compliance *= 2.0 - type_weight;
        }

        // Importance modifier (important directives get more attention)
        let importance_mod = 0.8 + directive.importance * 0.4;
        compliance *= importance_mod;

        // Urgency modifier (urgent directives may overcome some resistance)
        if directive.urgency > 0.7 && compliance > 0.0 {
            compliance *= 1.0 + (directive.urgency - 0.7) * 0.2;
        }

        // Personality trait modifier
        let trait_mod = Self::calculate_trait_modifier(
            &delegate.personality,
            &directive.directive_type,
            delegator,
            delegate,
        );
        compliance *= trait_mod;

        // Workload penalty (overworked delegates are less compliant)
        if delegate.workload > 0.8 {
            compliance *= 1.0 - (delegate.workload - 0.8) * 0.5;
        }

        // Add base compliance rate at the end
        compliance += config.base_compliance * 0.3;

        // Clamp to valid range
        compliance.clamp(-1.0, 1.0)
    }

    fn calculate_interpretation(_config: &DelegationConfig, input: &DelegationInput) -> f32 {
        let directive = &input.directive;
        let delegate = &input.delegate;

        // Base interpretation from directive type
        let mut interpretation = directive.directive_type.base_interpretation_freedom();

        // Personality adjustment
        interpretation *= match delegate.personality {
            DelegateTrait::Obedient => 0.5,
            DelegateTrait::Rigid => 0.3,
            DelegateTrait::Innovative => 1.5,
            DelegateTrait::Independent => 1.3,
            DelegateTrait::Eager => 1.1,
            DelegateTrait::Reluctant => 0.8,
            DelegateTrait::Loyal => 0.7,
            DelegateTrait::Opportunist => 1.2,
        };

        // Skill affects interpretation (skilled people may improve on orders)
        interpretation *= 0.8 + delegate.skill_level * 0.4;

        // Relationship affects interpretation (trusted delegates get more freedom)
        if delegate.relationship > 0.5 {
            interpretation *= 1.0 + (delegate.relationship - 0.5) * 0.3;
        }

        interpretation.clamp(0.0, 1.0)
    }

    fn calculate_priority(_config: &DelegationConfig, input: &DelegationInput) -> f32 {
        let directive = &input.directive;
        let delegator = &input.delegator;
        let delegate = &input.delegate;

        // Base priority from directive urgency and importance
        let mut priority = directive.urgency * 0.5 + directive.importance * 0.5;

        // Authority affects priority (higher authority = higher priority)
        priority *= 0.7 + delegator.authority * 0.5;

        // Relationship affects priority
        priority *= 0.8 + (delegate.relationship + 1.0) * 0.2; // Normalize -1..1 to 0..0.4

        // Personality adjustment
        priority *= match delegate.personality {
            DelegateTrait::Eager => 1.3,
            DelegateTrait::Loyal => 1.2,
            DelegateTrait::Obedient => 1.1,
            DelegateTrait::Reluctant => 0.7,
            DelegateTrait::Opportunist => 0.9,
            _ => 1.0,
        };

        // Task deadlines increase priority
        if let DirectiveType::Task { deadline: Some(dl), .. } = &directive.directive_type {
            let time_pressure = if *dl > input.current_tick {
                let remaining = (*dl - input.current_tick) as f32;
                (100.0 / (remaining + 10.0)).min(0.3) // Up to 0.3 bonus
            } else {
                0.3 // Overdue = max urgency
            };
            priority += time_pressure;
        }

        priority.clamp(0.0, 1.0)
    }

    fn calculate_feedback_probability(config: &DelegationConfig, input: &DelegationInput) -> f32 {
        let delegate = &input.delegate;
        let delegator = &input.delegator;

        // Base feedback rate
        let mut feedback = config.base_feedback_rate;

        // Relationship affects feedback (better relationship = more communication)
        feedback += delegate.relationship * 0.2;

        // Loyalty affects feedback
        feedback += delegate.loyalty * 0.15;

        // Personality affects feedback
        feedback *= match delegate.personality {
            DelegateTrait::Loyal => 1.3,
            DelegateTrait::Obedient => 1.2,
            DelegateTrait::Eager => 1.4,
            DelegateTrait::Independent => 0.8,
            DelegateTrait::Reluctant => 0.6,
            DelegateTrait::Opportunist => 0.7,
            _ => 1.0,
        };

        // Higher authority delegators get more feedback
        feedback *= 0.8 + delegator.authority * 0.4;

        feedback.clamp(0.0, 1.0)
    }

    fn calculate_expected_quality(config: &DelegationConfig, input: &DelegationInput) -> f32 {
        let delegate = &input.delegate;

        // Base quality from skill
        let mut quality = delegate.skill_level;

        // Workload penalty
        quality *= 1.0 - delegate.workload * 0.3;

        // Morale affects quality
        quality *= 0.7 + delegate.morale * 0.5;

        // Personality affects quality
        quality *= match delegate.personality {
            DelegateTrait::Eager => 1.15,
            DelegateTrait::Innovative => 1.1,
            DelegateTrait::Rigid => 0.95, // Consistent but not creative
            DelegateTrait::Reluctant => 0.7,
            DelegateTrait::Obedient => 1.0,
            _ => 1.0,
        };

        // Compliance affects quality (negative compliance = sabotage risk)
        let compliance = Self::calculate_compliance(config, input);
        if compliance < 0.0 {
            quality *= 1.0 + compliance * 0.5; // Negative compliance reduces quality
        } else {
            quality *= 0.8 + compliance * 0.3;
        }

        quality.clamp(0.0, 1.0)
    }

    fn calculate_propagation_delay(config: &DelegationConfig, input: &DelegationInput) -> f32 {
        let delegator = &input.delegator;
        let delegate = &input.delegate;
        let directive = &input.directive;

        // Base delay from hierarchy distance
        let hierarchy_distance = if delegate.hierarchy_rank > delegator.hierarchy_rank {
            (delegate.hierarchy_rank - delegator.hierarchy_rank) as f32
        } else {
            1.0
        };

        let mut delay = hierarchy_distance * config.propagation_delay_per_level;

        // Urgency reduces delay
        delay *= 1.0 - directive.urgency * 0.5;

        // Commands propagate faster than suggestions
        delay *= match &directive.directive_type {
            DirectiveType::Command { .. } => 0.7,
            DirectiveType::Request { .. } => 1.0,
            DirectiveType::Task { .. } => 0.9,
            DirectiveType::Suggestion { .. } => 1.3,
            DirectiveType::StandingOrder { .. } => 0.8,
        };

        // Personality affects response time
        delay *= match delegate.personality {
            DelegateTrait::Eager => 0.7,
            DelegateTrait::Obedient => 0.8,
            DelegateTrait::Reluctant => 1.5,
            DelegateTrait::Independent => 1.2,
            _ => 1.0,
        };

        delay.max(0.1)
    }

    fn calculate_trait_modifier(
        delegate_trait: &DelegateTrait,
        directive_type: &DirectiveType,
        delegator: &DelegatorStats,
        delegate: &DelegateStats,
    ) -> f32 {
        use DelegateTrait::*;

        match delegate_trait {
            Obedient => {
                // Always compliant, especially to commands
                match directive_type {
                    DirectiveType::Command { .. } => 1.3,
                    DirectiveType::StandingOrder { .. } => 1.2,
                    _ => 1.1,
                }
            }

            Independent => {
                // Questions everything, but respects good reasons
                // High authority/reputation helps
                let base = 0.8;
                let authority_bonus = delegator.authority * 0.2;
                let reputation_bonus = delegator.reputation.max(0.0) * 0.15;
                base + authority_bonus + reputation_bonus
            }

            Eager => {
                // Enthusiastic about everything
                1.2
            }

            Reluctant => {
                // Needs convincing
                let base = 0.6;
                // Urgent matters can overcome reluctance
                if let DirectiveType::Command { .. } = directive_type {
                    base + 0.2
                } else {
                    base
                }
            }

            Loyal => {
                // Very compliant when loyalty is high
                let loyalty_bonus = delegate.loyalty.max(0.0) * 0.4;
                0.9 + loyalty_bonus
            }

            Opportunist => {
                // Compliance depends on what's in it for them
                match directive_type {
                    DirectiveType::Request { compensation: Some(c), .. } => {
                        0.7 + c.min(0.5) // Compensation increases compliance
                    }
                    DirectiveType::Task { .. } => 0.9, // Tasks may bring advancement
                    _ => 0.75,
                }
            }

            Innovative => {
                // Prefers autonomy, less keen on rigid commands
                match directive_type {
                    DirectiveType::Command { .. } => 0.85,
                    DirectiveType::Suggestion { .. } => 1.2,
                    DirectiveType::Task { .. } => 1.1,
                    _ => 1.0,
                }
            }

            Rigid => {
                // Follows rules strictly
                match directive_type {
                    DirectiveType::Command { .. } => 1.25,
                    DirectiveType::StandingOrder { .. } => 1.2,
                    DirectiveType::Suggestion { .. } => 0.7, // Doesn't like vague guidance
                    _ => 1.0,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::delegation::types::{Directive, DirectiveId};

    fn create_test_input(
        loyalty: f32,
        relationship: f32,
        authority: f32,
        personality: DelegateTrait,
    ) -> DelegationInput {
        DelegationInput {
            directive: Directive {
                id: DirectiveId("test_directive".into()),
                directive_type: DirectiveType::Command {
                    target: "location".into(),
                    action: "move".into(),
                },
                urgency: 0.5,
                importance: 0.5,
                issued_at: 0,
            },
            delegator: DelegatorStats {
                entity_id: EntityId("commander".into()),
                authority,
                charisma: 0.5,
                hierarchy_rank: 0,
                reputation: 0.5,
            },
            delegate: DelegateStats {
                entity_id: EntityId("subordinate".into()),
                loyalty,
                morale: 0.7,
                relationship,
                hierarchy_rank: 1,
                personality,
                workload: 0.3,
                skill_level: 0.7,
            },
            current_tick: 100,
        }
    }

    use crate::mechanics::delegation::types::EntityId;

    #[test]
    fn test_compliance_loyal_obedient() {
        let config = DelegationConfig::default();
        let input = create_test_input(0.9, 0.8, 0.9, DelegateTrait::Obedient);

        let compliance = SimpleDelegationPolicy::calculate_compliance(&config, &input);

        // High loyalty, good relationship, high authority, obedient = very high compliance
        assert!(compliance > 0.8, "Expected high compliance, got {}", compliance);
    }

    #[test]
    fn test_compliance_disgruntled_independent() {
        let config = DelegationConfig::default();
        // More extreme values: negative loyalty, very poor relationship
        let input = create_test_input(-0.3, -0.5, 0.3, DelegateTrait::Independent);

        let compliance = SimpleDelegationPolicy::calculate_compliance(&config, &input);

        // Negative loyalty, poor relationship, low authority, independent = low compliance
        assert!(compliance < 0.5, "Expected low compliance, got {}", compliance);
    }

    #[test]
    fn test_compliance_negative_defiance() {
        let config = DelegationConfig::default();
        let input = create_test_input(-0.8, -0.9, 0.3, DelegateTrait::Independent);

        let compliance = SimpleDelegationPolicy::calculate_compliance(&config, &input);

        // Negative loyalty, terrible relationship, low authority = potential defiance
        assert!(compliance < 0.2, "Expected very low compliance, got {}", compliance);
    }

    #[test]
    fn test_interpretation_innovative_vs_rigid() {
        let config = DelegationConfig::default();

        let innovative_input = create_test_input(0.5, 0.5, 0.5, DelegateTrait::Innovative);
        let rigid_input = create_test_input(0.5, 0.5, 0.5, DelegateTrait::Rigid);

        let innovative_interp =
            SimpleDelegationPolicy::calculate_interpretation(&config, &innovative_input);
        let rigid_interp =
            SimpleDelegationPolicy::calculate_interpretation(&config, &rigid_input);

        assert!(
            innovative_interp > rigid_interp,
            "Innovative should have higher interpretation: {} vs {}",
            innovative_interp,
            rigid_interp
        );
    }

    #[test]
    fn test_priority_eager_vs_reluctant() {
        let config = DelegationConfig::default();

        let eager_input = create_test_input(0.5, 0.5, 0.5, DelegateTrait::Eager);
        let reluctant_input = create_test_input(0.5, 0.5, 0.5, DelegateTrait::Reluctant);

        let eager_priority = SimpleDelegationPolicy::calculate_priority(&config, &eager_input);
        let reluctant_priority =
            SimpleDelegationPolicy::calculate_priority(&config, &reluctant_input);

        assert!(
            eager_priority > reluctant_priority,
            "Eager should have higher priority: {} vs {}",
            eager_priority,
            reluctant_priority
        );
    }

    #[test]
    fn test_feedback_loyal_vs_independent() {
        let config = DelegationConfig::default();

        let loyal_input = create_test_input(0.9, 0.8, 0.5, DelegateTrait::Loyal);
        let independent_input = create_test_input(0.5, 0.3, 0.5, DelegateTrait::Independent);

        let loyal_feedback =
            SimpleDelegationPolicy::calculate_feedback_probability(&config, &loyal_input);
        let independent_feedback =
            SimpleDelegationPolicy::calculate_feedback_probability(&config, &independent_input);

        assert!(
            loyal_feedback > independent_feedback,
            "Loyal should give more feedback: {} vs {}",
            loyal_feedback,
            independent_feedback
        );
    }

    #[test]
    fn test_propagation_delay_urgency() {
        let config = DelegationConfig::default();

        let mut normal_input = create_test_input(0.5, 0.5, 0.5, DelegateTrait::Obedient);
        normal_input.directive.urgency = 0.3;

        let mut urgent_input = create_test_input(0.5, 0.5, 0.5, DelegateTrait::Obedient);
        urgent_input.directive.urgency = 0.9;

        let normal_delay =
            SimpleDelegationPolicy::calculate_propagation_delay(&config, &normal_input);
        let urgent_delay =
            SimpleDelegationPolicy::calculate_propagation_delay(&config, &urgent_input);

        assert!(
            urgent_delay < normal_delay,
            "Urgent should have shorter delay: {} vs {}",
            urgent_delay,
            normal_delay
        );
    }

    #[test]
    fn test_trait_modifier_opportunist_compensation() {
        let delegator = DelegatorStats {
            entity_id: EntityId("boss".into()),
            authority: 0.7,
            charisma: 0.5,
            hierarchy_rank: 0,
            reputation: 0.5,
        };
        let delegate = DelegateStats {
            entity_id: EntityId("worker".into()),
            loyalty: 0.5,
            morale: 0.7,
            relationship: 0.5,
            hierarchy_rank: 1,
            personality: DelegateTrait::Opportunist,
            workload: 0.3,
            skill_level: 0.7,
        };

        let with_comp = SimpleDelegationPolicy::calculate_trait_modifier(
            &DelegateTrait::Opportunist,
            &DirectiveType::Request {
                description: "help".into(),
                compensation: Some(0.5),
            },
            &delegator,
            &delegate,
        );

        let without_comp = SimpleDelegationPolicy::calculate_trait_modifier(
            &DelegateTrait::Opportunist,
            &DirectiveType::Request {
                description: "help".into(),
                compensation: None,
            },
            &delegator,
            &delegate,
        );

        assert!(
            with_comp > without_comp,
            "Compensation should increase opportunist modifier: {} vs {}",
            with_comp,
            without_comp
        );
    }
}
