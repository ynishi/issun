//! Simple organization policy implementation.

use crate::mechanics::organization::policies::OrganizationPolicy;
use crate::mechanics::organization::types::{
    MemberArchetype, MemberId, OrganizationConfig, OrganizationInput, OrganizationState,
    OrganizationType,
};
use std::collections::HashMap;

/// Simple organization policy based on straightforward calculations
///
/// This policy implements intuitive organizational dynamics:
/// - **Decision Speed**: Base speed modified by member count, urgency, and charisma
/// - **Consensus**: Based on org type with importance modifier
/// - **Authority**: Power-law or flat distribution based on org type
/// - **Loyalty**: Archetype-org fit matrix
///
/// # Borderlands Example
///
/// ```ignore
/// // Bandit Cult (e.g., Children of the Vault)
/// let input = OrganizationInput {
///     org_type: OrganizationType::Cult,
///     leader_charisma: 0.95, // Charismatic psycho leader
///     member_count: 200,
///     urgency: 0.8,
///     ..
/// };
/// // Result: Very fast decisions, zero consensus needed, extreme loyalty from Devotees
///
/// // Hyperion Corporation
/// let input = OrganizationInput {
///     org_type: OrganizationType::Corporate,
///     leader_charisma: 0.6, // Handsome Jack
///     member_count: 10000,
///     urgency: 0.5,
///     ..
/// };
/// // Result: Fast executive decisions, concentrated authority, ambitious members thrive
/// ```
pub struct SimpleOrganizationPolicy;

impl OrganizationPolicy for SimpleOrganizationPolicy {
    fn calculate_decision_speed(config: &OrganizationConfig, input: &OrganizationInput) -> f32 {
        // Get base speed for org type
        let base_speed = config
            .base_decision_speeds
            .get(&input.org_type)
            .copied()
            .unwrap_or(1.0);

        // Member count penalty (more members = slower, except for Cult)
        let member_penalty = if matches!(input.org_type, OrganizationType::Cult) {
            // Cults scale with charismatic leadership, not member count
            1.0
        } else if matches!(input.org_type, OrganizationType::Anarchy) {
            // Anarchy gets worse with more members
            1.0 / (1.0 + (input.member_count as f32 * config.member_count_scaling).sqrt())
        } else {
            // Standard logarithmic scaling
            1.0 / (1.0 + (input.member_count as f32 / 100.0).ln().max(0.0) * config.member_count_scaling)
        };

        // Urgency bonus
        let urgency_boost = 1.0 + input.urgency * config.urgency_bonus;

        // Charisma bonus (primarily for Cult, but affects all)
        let charisma_bonus = match input.org_type {
            OrganizationType::Cult => 1.0 + input.leader_charisma * config.charisma_influence,
            OrganizationType::Hierarchy | OrganizationType::Corporate => {
                1.0 + input.leader_charisma * config.charisma_influence * 0.3
            }
            _ => 1.0,
        };

        // Calculate final speed
        let speed = base_speed * member_penalty * urgency_boost * charisma_bonus;

        // Clamp to configured bounds
        speed.clamp(config.min_decision_speed, config.max_decision_speed)
    }

    fn calculate_consensus_requirement(
        config: &OrganizationConfig,
        input: &OrganizationInput,
    ) -> f32 {
        // Get base threshold for org type
        let base_threshold = config
            .consensus_thresholds
            .get(&input.org_type)
            .copied()
            .unwrap_or(0.5);

        // Important decisions require more consensus (except in autocratic orgs)
        let importance_modifier = match input.org_type {
            OrganizationType::Cult => 0.0, // Leader decides, importance doesn't matter
            OrganizationType::Hierarchy => input.decision_importance * 0.1,
            OrganizationType::Corporate => input.decision_importance * 0.15,
            _ => input.decision_importance * 0.2,
        };

        (base_threshold + importance_modifier).clamp(0.0, 1.0)
    }

    fn determine_authority_distribution(
        config: &OrganizationConfig,
        input: &OrganizationInput,
    ) -> HashMap<MemberId, f32> {
        let members: Vec<&MemberId> = input.member_archetypes.iter().map(|(id, _)| id).collect();
        let count = members.len();

        if count == 0 {
            return HashMap::new();
        }

        let weights: Vec<f32> = match input.org_type {
            OrganizationType::Cult => {
                // Leader (first member) has ALL power
                let mut w = vec![0.0; count];
                w[0] = 1.0;
                w
            }

            OrganizationType::Hierarchy | OrganizationType::Corporate => {
                // Power-law distribution: top has most power
                let concentration = config.authority_concentration;
                let mut w: Vec<f32> = (0..count)
                    .map(|i| 1.0 / (i as f32 + 1.0).powf(1.0 + concentration))
                    .collect();
                normalize_weights(&mut w);
                w
            }

            OrganizationType::Democracy | OrganizationType::Holacracy => {
                // Flat distribution (equal power)
                vec![1.0 / count as f32; count]
            }

            OrganizationType::Tribal => {
                // Elders (top 20%) have 80% of power
                let elder_count = (count as f32 * 0.2).ceil() as usize;
                let elder_count = elder_count.max(1);
                let other_count = count - elder_count;

                let mut w = Vec::with_capacity(count);
                let elder_share = 0.8 / elder_count as f32;
                let other_share = if other_count > 0 {
                    0.2 / other_count as f32
                } else {
                    0.0
                };

                for i in 0..count {
                    if i < elder_count {
                        w.push(elder_share);
                    } else {
                        w.push(other_share);
                    }
                }
                w
            }

            OrganizationType::Anarchy => {
                // No formal authority (all equal, but low)
                vec![1.0 / count as f32; count]
            }
        };

        // Build HashMap
        members
            .into_iter()
            .zip(weights.into_iter())
            .map(|(id, w)| (id.clone(), w))
            .collect()
    }

    fn calculate_loyalty_modifier(org_type: &OrganizationType, archetype: &MemberArchetype) -> f32 {
        use MemberArchetype::*;
        use OrganizationType::*;

        match (org_type, archetype) {
            // Excellent fits (>1.3)
            (Cult, Devotee) => 1.5,
            (Hierarchy, Authoritarian) => 1.3,
            (Democracy, Egalitarian) => 1.3,
            (Holacracy, Autonomous) => 1.4,
            (Tribal, Traditionalist) => 1.4,
            (Corporate, Ambitious) => 1.35,
            (Anarchy, Autonomous) => 1.4,
            (Anarchy, Egalitarian) => 1.3,

            // Good fits (1.1-1.3)
            (Hierarchy, Ambitious) => 1.2,
            (Corporate, Authoritarian) => 1.15,
            (Holacracy, Egalitarian) => 1.2,
            (Democracy, Autonomous) => 1.1,
            (Tribal, Devotee) => 1.1,

            // Neutral (Pragmatic fits anywhere)
            (_, Pragmatic) => 1.0,

            // Poor fits (0.7-0.9)
            (Cult, Autonomous) => 0.4,
            (Cult, Egalitarian) => 0.5,
            (Hierarchy, Egalitarian) => 0.75,
            (Hierarchy, Autonomous) => 0.8,
            (Democracy, Authoritarian) => 0.8,
            (Anarchy, Authoritarian) => 0.6,
            (Corporate, Egalitarian) => 0.85,
            (Tribal, Ambitious) => 0.7,

            // Default neutral
            _ => 0.9,
        }
    }

    fn calculate_efficiency(
        _config: &OrganizationConfig,
        state: &OrganizationState,
        input: &OrganizationInput,
    ) -> f32 {
        // Base efficiency
        let mut efficiency = 0.8;

        // Cohesion bonus/penalty
        efficiency *= 0.5 + state.cohesion * 0.5;

        // Size efficiency (optimal around 7-15 members, penalties for too small/large)
        let size_efficiency = match input.member_count {
            0..=2 => 0.6,
            3..=6 => 0.85,
            7..=15 => 1.0,
            16..=50 => 0.95,
            51..=200 => 0.85,
            _ => 0.7,
        };
        efficiency *= size_efficiency;

        // Urgency stress penalty (constant high urgency = burnout)
        if input.urgency > 0.8 {
            efficiency *= 1.0 - (input.urgency - 0.8) * 0.5;
        }

        // Org type efficiency characteristics
        let type_efficiency = match input.org_type {
            OrganizationType::Hierarchy => 0.9,  // Good at execution
            OrganizationType::Corporate => 0.85, // Bureaucracy overhead
            OrganizationType::Holacracy => 0.95, // Efficient self-organization
            OrganizationType::Democracy => 0.8,  // Deliberation overhead
            OrganizationType::Cult => {
                // Cult efficiency depends heavily on leader charisma
                0.7 + input.leader_charisma * 0.3
            }
            OrganizationType::Tribal => 0.85,
            OrganizationType::Anarchy => 0.6, // Coordination problems
        };
        efficiency *= type_efficiency;

        efficiency.clamp(0.1, 1.0)
    }

    fn calculate_cohesion(loyalty_modifiers: &HashMap<MemberId, f32>) -> f32 {
        if loyalty_modifiers.is_empty() {
            return 0.5;
        }

        // Average loyalty modifier indicates cohesion
        let sum: f32 = loyalty_modifiers.values().sum();
        let avg = sum / loyalty_modifiers.len() as f32;

        // Also consider variance (high variance = low cohesion)
        let variance: f32 = loyalty_modifiers
            .values()
            .map(|&m| (m - avg).powi(2))
            .sum::<f32>()
            / loyalty_modifiers.len() as f32;

        // High average loyalty + low variance = high cohesion
        let cohesion = avg * (1.0 - variance.sqrt().min(0.5) * 2.0);

        cohesion.clamp(0.0, 1.0)
    }
}

/// Normalize weights to sum to 1.0
fn normalize_weights(weights: &mut [f32]) {
    let sum: f32 = weights.iter().sum();
    if sum > 0.0 {
        for w in weights.iter_mut() {
            *w /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_input(org_type: OrganizationType, member_count: usize) -> OrganizationInput {
        let members: Vec<(MemberId, MemberArchetype)> = (0..member_count)
            .map(|i| (MemberId(format!("m{}", i)), MemberArchetype::Pragmatic))
            .collect();

        OrganizationInput {
            org_type,
            member_count,
            decision_importance: 0.5,
            urgency: 0.5,
            leader_charisma: 0.7,
            member_archetypes: members,
            current_tick: 0,
        }
    }

    #[test]
    fn test_decision_speed_cult_fastest() {
        let config = OrganizationConfig::default();
        let cult_input = create_input(OrganizationType::Cult, 50);
        let democracy_input = create_input(OrganizationType::Democracy, 50);

        let cult_speed = SimpleOrganizationPolicy::calculate_decision_speed(&config, &cult_input);
        let dem_speed =
            SimpleOrganizationPolicy::calculate_decision_speed(&config, &democracy_input);

        assert!(cult_speed > dem_speed);
        assert!(cult_speed > 2.0); // Cult with charisma should be > 2.0
    }

    #[test]
    fn test_decision_speed_urgency_boost() {
        let config = OrganizationConfig::default();
        let mut low_urgency = create_input(OrganizationType::Hierarchy, 20);
        low_urgency.urgency = 0.1;
        let mut high_urgency = create_input(OrganizationType::Hierarchy, 20);
        high_urgency.urgency = 0.9;

        let low_speed =
            SimpleOrganizationPolicy::calculate_decision_speed(&config, &low_urgency);
        let high_speed =
            SimpleOrganizationPolicy::calculate_decision_speed(&config, &high_urgency);

        assert!(high_speed > low_speed);
    }

    #[test]
    fn test_consensus_cult_zero() {
        let config = OrganizationConfig::default();
        let input = create_input(OrganizationType::Cult, 100);

        let consensus =
            SimpleOrganizationPolicy::calculate_consensus_requirement(&config, &input);

        assert_eq!(consensus, 0.0); // Cult leader decides alone
    }

    #[test]
    fn test_consensus_democracy_majority() {
        let config = OrganizationConfig::default();
        let input = create_input(OrganizationType::Democracy, 100);

        let consensus =
            SimpleOrganizationPolicy::calculate_consensus_requirement(&config, &input);

        assert!(consensus > 0.5); // Democracy needs majority+
    }

    #[test]
    fn test_authority_cult_concentrated() {
        let config = OrganizationConfig::default();
        let mut input = create_input(OrganizationType::Cult, 5);
        input.member_archetypes = vec![
            (MemberId("leader".into()), MemberArchetype::Devotee),
            (MemberId("m1".into()), MemberArchetype::Devotee),
            (MemberId("m2".into()), MemberArchetype::Devotee),
            (MemberId("m3".into()), MemberArchetype::Devotee),
            (MemberId("m4".into()), MemberArchetype::Devotee),
        ];

        let distribution =
            SimpleOrganizationPolicy::determine_authority_distribution(&config, &input);

        // Leader should have ALL authority
        assert_eq!(distribution.get(&MemberId("leader".into())), Some(&1.0));
        assert_eq!(distribution.get(&MemberId("m1".into())), Some(&0.0));
    }

    #[test]
    fn test_authority_democracy_flat() {
        let config = OrganizationConfig::default();
        let mut input = create_input(OrganizationType::Democracy, 4);
        input.member_archetypes = vec![
            (MemberId("m0".into()), MemberArchetype::Egalitarian),
            (MemberId("m1".into()), MemberArchetype::Egalitarian),
            (MemberId("m2".into()), MemberArchetype::Egalitarian),
            (MemberId("m3".into()), MemberArchetype::Egalitarian),
        ];

        let distribution =
            SimpleOrganizationPolicy::determine_authority_distribution(&config, &input);

        // All should have equal authority
        for (_, weight) in &distribution {
            assert!((weight - 0.25).abs() < 0.01);
        }
    }

    #[test]
    fn test_loyalty_modifier_excellent_fit() {
        let modifier = SimpleOrganizationPolicy::calculate_loyalty_modifier(
            &OrganizationType::Cult,
            &MemberArchetype::Devotee,
        );
        assert_eq!(modifier, 1.5); // Devotee in Cult = excellent
    }

    #[test]
    fn test_loyalty_modifier_terrible_fit() {
        let modifier = SimpleOrganizationPolicy::calculate_loyalty_modifier(
            &OrganizationType::Cult,
            &MemberArchetype::Autonomous,
        );
        assert_eq!(modifier, 0.4); // Autonomous in Cult = terrible
    }

    #[test]
    fn test_loyalty_modifier_pragmatic_always_neutral() {
        for org_type in [
            OrganizationType::Cult,
            OrganizationType::Democracy,
            OrganizationType::Hierarchy,
            OrganizationType::Anarchy,
        ] {
            let modifier = SimpleOrganizationPolicy::calculate_loyalty_modifier(
                &org_type,
                &MemberArchetype::Pragmatic,
            );
            assert_eq!(modifier, 1.0);
        }
    }

    #[test]
    fn test_efficiency_holacracy_highest() {
        let config = OrganizationConfig::default();
        let state = OrganizationState::default();

        let hol_input = create_input(OrganizationType::Holacracy, 10);
        let anarchy_input = create_input(OrganizationType::Anarchy, 10);

        let hol_eff =
            SimpleOrganizationPolicy::calculate_efficiency(&config, &state, &hol_input);
        let anarchy_eff =
            SimpleOrganizationPolicy::calculate_efficiency(&config, &state, &anarchy_input);

        assert!(hol_eff > anarchy_eff);
    }

    #[test]
    fn test_cohesion_uniform_high() {
        let mut modifiers = HashMap::new();
        modifiers.insert(MemberId("m1".into()), 1.2);
        modifiers.insert(MemberId("m2".into()), 1.2);
        modifiers.insert(MemberId("m3".into()), 1.2);

        let cohesion = SimpleOrganizationPolicy::calculate_cohesion(&modifiers);
        assert!(cohesion > 0.8); // Uniform high loyalty = high cohesion
    }

    #[test]
    fn test_cohesion_mixed_lower() {
        let mut modifiers = HashMap::new();
        modifiers.insert(MemberId("m1".into()), 1.5);
        modifiers.insert(MemberId("m2".into()), 0.5);
        modifiers.insert(MemberId("m3".into()), 1.0);

        let cohesion = SimpleOrganizationPolicy::calculate_cohesion(&modifiers);
        assert!(cohesion < 0.8); // Mixed loyalty = lower cohesion
    }
}
