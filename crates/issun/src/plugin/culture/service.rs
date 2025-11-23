//! Pure logic service for culture management
//!
//! This service provides stateless functions for alignment checking,
//! stress/fervor calculation, and culture effect determination.

use super::config::CultureConfig;
use super::types::{Alignment, CultureEffect, CultureTag, Member, PersonalityTrait};
use std::collections::HashSet;

/// Culture service (stateless, pure functions)
///
/// All methods are pure functions with no side effects, making them easy to test.
#[derive(Debug, Clone, Copy, Default)]
pub struct CultureService;

impl CultureService {
    /// Check alignment between member personality and organization culture
    ///
    /// # Arguments
    ///
    /// * `member` - The member to check
    /// * `culture_tags` - Organization's culture tags
    ///
    /// # Returns
    ///
    /// Alignment indicating whether member fits the culture
    ///
    /// # Alignment Rules
    ///
    /// - **Aligned**: Personality matches culture (fervor bonus)
    ///   - Cautious + Bureaucratic
    ///   - Bold + RiskTaking
    ///   - Zealous + Fanatic
    ///   - Workaholic + Overwork
    ///   - Collaborative + PsychologicalSafety
    ///   - Competitive + Ruthless
    ///
    /// - **Misaligned**: Personality conflicts with culture (stress accumulation)
    ///   - Cautious + RiskTaking
    ///   - Bold + Bureaucratic
    ///   - Pragmatic + Fanatic
    ///   - Balanced + Overwork
    ///   - Collaborative + Ruthless
    ///
    /// - **Neutral**: No strong match or conflict
    pub fn check_alignment(
        member: &Member,
        culture_tags: &HashSet<CultureTag>,
    ) -> Alignment {
        let personality_traits = &member.personality_traits;

        // Check for alignment (positive matches)
        if personality_traits.contains(&PersonalityTrait::Cautious)
            && culture_tags.contains(&CultureTag::Bureaucratic)
        {
            return Alignment::Aligned { fervor_bonus: 0.05 };
        }

        if personality_traits.contains(&PersonalityTrait::Bold)
            && culture_tags.contains(&CultureTag::RiskTaking)
        {
            return Alignment::Aligned { fervor_bonus: 0.06 };
        }

        if personality_traits.contains(&PersonalityTrait::Zealous)
            && culture_tags.contains(&CultureTag::Fanatic)
        {
            return Alignment::Aligned { fervor_bonus: 0.08 };
        }

        if personality_traits.contains(&PersonalityTrait::Workaholic)
            && culture_tags.contains(&CultureTag::Overwork)
        {
            return Alignment::Aligned { fervor_bonus: 0.05 };
        }

        if personality_traits.contains(&PersonalityTrait::Collaborative)
            && culture_tags.contains(&CultureTag::PsychologicalSafety)
        {
            return Alignment::Aligned { fervor_bonus: 0.04 };
        }

        if personality_traits.contains(&PersonalityTrait::Competitive)
            && culture_tags.contains(&CultureTag::Ruthless)
        {
            return Alignment::Aligned { fervor_bonus: 0.05 };
        }

        // Check for misalignment (conflicts)
        if personality_traits.contains(&PersonalityTrait::Cautious)
            && culture_tags.contains(&CultureTag::RiskTaking)
        {
            return Alignment::Misaligned {
                stress_rate: 0.08,
                reason: "Cautious personality in risk-taking culture".to_string(),
            };
        }

        if personality_traits.contains(&PersonalityTrait::Bold)
            && culture_tags.contains(&CultureTag::Bureaucratic)
        {
            return Alignment::Misaligned {
                stress_rate: 0.06,
                reason: "Bold personality in bureaucratic culture".to_string(),
            };
        }

        if personality_traits.contains(&PersonalityTrait::Pragmatic)
            && culture_tags.contains(&CultureTag::Fanatic)
        {
            return Alignment::Misaligned {
                stress_rate: 0.10,
                reason: "Pragmatic personality in fanatic culture".to_string(),
            };
        }

        if personality_traits.contains(&PersonalityTrait::Balanced)
            && culture_tags.contains(&CultureTag::Overwork)
        {
            return Alignment::Misaligned {
                stress_rate: 0.07,
                reason: "Balanced personality in overwork culture".to_string(),
            };
        }

        if personality_traits.contains(&PersonalityTrait::Collaborative)
            && culture_tags.contains(&CultureTag::Ruthless)
        {
            return Alignment::Misaligned {
                stress_rate: 0.09,
                reason: "Collaborative personality in ruthless culture".to_string(),
            };
        }

        // No strong match or conflict
        Alignment::Neutral
    }

    /// Calculate stress change for a member
    ///
    /// # Formula
    ///
    /// - **Aligned**: No stress change (or slight decrease)
    /// - **Misaligned**: `stress_change = alignment.stress_rate * config.base_stress_rate * culture_strength`
    /// - **Neutral**: Minimal stress decay if enabled
    ///
    /// # Arguments
    ///
    /// * `current_stress` - Current stress level (0.0-1.0)
    /// * `alignment` - Member's alignment with culture
    /// * `config` - Configuration
    /// * `culture_strength` - Culture strength multiplier (0.0-2.0)
    ///
    /// # Returns
    ///
    /// New stress level (clamped to 0.0-1.0)
    pub fn calculate_stress_change(
        current_stress: f32,
        alignment: &Alignment,
        config: &CultureConfig,
        culture_strength: f32,
    ) -> f32 {
        let new_stress = match alignment {
            Alignment::Aligned { .. } => {
                // Aligned members recover from stress faster
                if config.enable_stress_decay {
                    (current_stress - config.stress_decay_rate * 2.0).max(0.0)
                } else {
                    current_stress
                }
            }
            Alignment::Misaligned { stress_rate, .. } => {
                // Misaligned members accumulate stress
                let stress_increase = stress_rate * config.base_stress_rate * culture_strength;
                current_stress + stress_increase
            }
            Alignment::Neutral => {
                // Neutral members have slow stress decay
                if config.enable_stress_decay {
                    (current_stress - config.stress_decay_rate).max(0.0)
                } else {
                    current_stress
                }
            }
        };

        new_stress.clamp(0.0, 1.0)
    }

    /// Calculate fervor change for a member
    ///
    /// # Formula
    ///
    /// - **Aligned**: `fervor_change = alignment.fervor_bonus * config.base_fervor_growth_rate * culture_strength`
    /// - **Misaligned**: Fervor decreases slowly
    /// - **Neutral**: No fervor change
    ///
    /// # Arguments
    ///
    /// * `current_fervor` - Current fervor level (0.0-1.0)
    /// * `alignment` - Member's alignment with culture
    /// * `config` - Configuration
    /// * `culture_strength` - Culture strength multiplier (0.0-2.0)
    ///
    /// # Returns
    ///
    /// New fervor level (clamped to 0.0-1.0)
    pub fn calculate_fervor_change(
        current_fervor: f32,
        alignment: &Alignment,
        config: &CultureConfig,
        culture_strength: f32,
    ) -> f32 {
        let new_fervor = match alignment {
            Alignment::Aligned { fervor_bonus } => {
                // Aligned members gain fervor
                let fervor_increase = fervor_bonus * config.base_fervor_growth_rate * culture_strength;
                current_fervor + fervor_increase
            }
            Alignment::Misaligned { .. } => {
                // Misaligned members lose fervor slowly
                (current_fervor - 0.01).max(0.0)
            }
            Alignment::Neutral => {
                // Neutral members maintain fervor
                current_fervor
            }
        };

        new_fervor.clamp(0.0, 1.0)
    }

    /// Check if member is stressed out (breakdown risk)
    ///
    /// # Arguments
    ///
    /// * `member` - Member to check
    /// * `config` - Configuration with stress threshold
    ///
    /// # Returns
    ///
    /// `true` if stress exceeds breakdown threshold
    pub fn is_stressed_out(member: &Member, config: &CultureConfig) -> bool {
        member.stress >= config.stress_breakdown_threshold
    }

    /// Check if member is fanatical
    ///
    /// # Arguments
    ///
    /// * `member` - Member to check
    /// * `config` - Configuration with fervor threshold
    ///
    /// # Returns
    ///
    /// `true` if fervor exceeds fanaticism threshold
    pub fn is_fanatical(member: &Member, config: &CultureConfig) -> bool {
        member.fervor >= config.fervor_fanaticism_threshold
    }

    /// Get culture effects from culture tags
    ///
    /// # Arguments
    ///
    /// * `culture_tags` - Organization's culture tags
    /// * `culture_strength` - Culture strength multiplier
    ///
    /// # Returns
    ///
    /// Vector of culture effects to apply
    pub fn get_culture_effects(
        culture_tags: &HashSet<CultureTag>,
        culture_strength: f32,
    ) -> Vec<CultureEffect> {
        let mut effects = Vec::new();

        for tag in culture_tags {
            match tag {
                CultureTag::RiskTaking => {
                    effects.push(CultureEffect::IncreasedRiskTaking {
                        magnitude: 0.3 * culture_strength,
                    });
                }
                CultureTag::PsychologicalSafety => {
                    effects.push(CultureEffect::ImprovedReporting {
                        magnitude: 0.4 * culture_strength,
                    });
                }
                CultureTag::Ruthless => {
                    effects.push(CultureEffect::IncreasedCompetition {
                        magnitude: 0.5 * culture_strength,
                    });
                }
                CultureTag::Bureaucratic => {
                    effects.push(CultureEffect::IncreasedCaution {
                        magnitude: 0.4 * culture_strength,
                    });
                }
                CultureTag::Fanatic => {
                    effects.push(CultureEffect::Fearless);
                }
                CultureTag::Overwork => {
                    effects.push(CultureEffect::IncreasedProductivity {
                        stress_cost: 0.05 * culture_strength,
                    });
                }
                CultureTag::Martyrdom => {
                    effects.push(CultureEffect::SelfSacrifice {
                        probability: 0.2 * culture_strength,
                    });
                }
                CultureTag::Custom(_) => {
                    // Custom tags don't have predefined effects
                }
            }
        }

        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_member_with_trait(trait_: PersonalityTrait) -> Member {
        Member::new("m1", "Test").with_trait(trait_)
    }

    fn create_culture_tags(tags: Vec<CultureTag>) -> HashSet<CultureTag> {
        tags.into_iter().collect()
    }

    #[test]
    fn test_alignment_cautious_bureaucratic() {
        let member = create_member_with_trait(PersonalityTrait::Cautious);
        let tags = create_culture_tags(vec![CultureTag::Bureaucratic]);

        let alignment = CultureService::check_alignment(&member, &tags);

        match alignment {
            Alignment::Aligned { fervor_bonus } => {
                assert_eq!(fervor_bonus, 0.05);
            }
            _ => panic!("Expected Aligned"),
        }
    }

    #[test]
    fn test_alignment_bold_risk_taking() {
        let member = create_member_with_trait(PersonalityTrait::Bold);
        let tags = create_culture_tags(vec![CultureTag::RiskTaking]);

        let alignment = CultureService::check_alignment(&member, &tags);

        match alignment {
            Alignment::Aligned { fervor_bonus } => {
                assert_eq!(fervor_bonus, 0.06);
            }
            _ => panic!("Expected Aligned"),
        }
    }

    #[test]
    fn test_misalignment_cautious_risk_taking() {
        let member = create_member_with_trait(PersonalityTrait::Cautious);
        let tags = create_culture_tags(vec![CultureTag::RiskTaking]);

        let alignment = CultureService::check_alignment(&member, &tags);

        match alignment {
            Alignment::Misaligned { stress_rate, .. } => {
                assert_eq!(stress_rate, 0.08);
            }
            _ => panic!("Expected Misaligned"),
        }
    }

    #[test]
    fn test_alignment_neutral() {
        let member = create_member_with_trait(PersonalityTrait::Balanced);
        let tags = create_culture_tags(vec![CultureTag::PsychologicalSafety]);

        let alignment = CultureService::check_alignment(&member, &tags);

        assert_eq!(alignment, Alignment::Neutral);
    }

    #[test]
    fn test_stress_increase_misaligned() {
        let config = CultureConfig::default();
        let alignment = Alignment::Misaligned {
            stress_rate: 0.08,
            reason: "Test".to_string(),
        };

        let new_stress = CultureService::calculate_stress_change(
            0.5,
            &alignment,
            &config,
            1.0,
        );

        // stress_increase = 0.08 * 0.03 * 1.0 = 0.0024
        // new_stress = 0.5 + 0.0024 = 0.5024
        assert!(new_stress > 0.5);
        assert!(new_stress < 0.51);
    }

    #[test]
    fn test_stress_decrease_aligned() {
        let config = CultureConfig::default();
        let alignment = Alignment::Aligned { fervor_bonus: 0.05 };

        let new_stress = CultureService::calculate_stress_change(
            0.5,
            &alignment,
            &config,
            1.0,
        );

        // Aligned members recover stress faster
        // new_stress = 0.5 - (0.01 * 2.0) = 0.48
        assert!(new_stress < 0.5);
    }

    #[test]
    fn test_fervor_increase_aligned() {
        let config = CultureConfig::default();
        let alignment = Alignment::Aligned { fervor_bonus: 0.06 };

        let new_fervor = CultureService::calculate_fervor_change(
            0.5,
            &alignment,
            &config,
            1.0,
        );

        // fervor_increase = 0.06 * 0.02 * 1.0 = 0.0012
        // new_fervor = 0.5 + 0.0012 = 0.5012
        assert!(new_fervor > 0.5);
        assert!(new_fervor < 0.51);
    }

    #[test]
    fn test_fervor_decrease_misaligned() {
        let config = CultureConfig::default();
        let alignment = Alignment::Misaligned {
            stress_rate: 0.08,
            reason: "Test".to_string(),
        };

        let new_fervor = CultureService::calculate_fervor_change(
            0.5,
            &alignment,
            &config,
            1.0,
        );

        // new_fervor = 0.5 - 0.01 = 0.49
        assert_eq!(new_fervor, 0.49);
    }

    #[test]
    fn test_is_stressed_out() {
        let config = CultureConfig::default();
        let member = Member::new("m1", "Test").with_stress(0.85);

        assert!(CultureService::is_stressed_out(&member, &config));
    }

    #[test]
    fn test_is_not_stressed_out() {
        let config = CultureConfig::default();
        let member = Member::new("m1", "Test").with_stress(0.5);

        assert!(!CultureService::is_stressed_out(&member, &config));
    }

    #[test]
    fn test_is_fanatical() {
        let config = CultureConfig::default();
        let member = Member::new("m1", "Test").with_fervor(0.95);

        assert!(CultureService::is_fanatical(&member, &config));
    }

    #[test]
    fn test_is_not_fanatical() {
        let config = CultureConfig::default();
        let member = Member::new("m1", "Test").with_fervor(0.7);

        assert!(!CultureService::is_fanatical(&member, &config));
    }

    #[test]
    fn test_get_culture_effects_risk_taking() {
        let tags = create_culture_tags(vec![CultureTag::RiskTaking]);
        let effects = CultureService::get_culture_effects(&tags, 1.0);

        assert_eq!(effects.len(), 1);
        match &effects[0] {
            CultureEffect::IncreasedRiskTaking { magnitude } => {
                assert_eq!(*magnitude, 0.3);
            }
            _ => panic!("Expected IncreasedRiskTaking"),
        }
    }

    #[test]
    fn test_get_culture_effects_multiple() {
        let tags = create_culture_tags(vec![
            CultureTag::RiskTaking,
            CultureTag::PsychologicalSafety,
            CultureTag::Fanatic,
        ]);
        let effects = CultureService::get_culture_effects(&tags, 1.0);

        assert_eq!(effects.len(), 3);
    }

    #[test]
    fn test_get_culture_effects_with_strength() {
        let tags = create_culture_tags(vec![CultureTag::RiskTaking]);
        let effects = CultureService::get_culture_effects(&tags, 2.0);

        match &effects[0] {
            CultureEffect::IncreasedRiskTaking { magnitude } => {
                assert_eq!(*magnitude, 0.6); // 0.3 * 2.0
            }
            _ => panic!("Expected IncreasedRiskTaking"),
        }
    }

    #[test]
    fn test_stress_clamping() {
        let config = CultureConfig::default();
        let alignment = Alignment::Misaligned {
            stress_rate: 1.0, // Very high
            reason: "Test".to_string(),
        };

        let new_stress = CultureService::calculate_stress_change(
            0.95,
            &alignment,
            &config,
            10.0, // Very high culture strength
        );

        // Should clamp to 1.0
        assert_eq!(new_stress, 1.0);
    }

    #[test]
    fn test_fervor_clamping() {
        let config = CultureConfig::default();
        let alignment = Alignment::Aligned { fervor_bonus: 1.0 };

        let new_fervor = CultureService::calculate_fervor_change(
            0.95,
            &alignment,
            &config,
            10.0,
        );

        // Should clamp to 1.0
        assert_eq!(new_fervor, 1.0);
    }
}
