//! Core data types for CulturePlugin

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Unique identifier for a member
pub type MemberId = String;

/// Unique identifier for a faction/organization
pub type FactionId = String;

/// Culture tags representing organizational "atmosphere" and implicit rules
///
/// These tags define the memetic DNA of an organization - the unwritten rules
/// that guide behavior more powerfully than explicit commands.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CultureTag {
    /// Risk-taking culture (encourages innovation but increases accidents)
    RiskTaking,

    /// Psychological safety (encourages reporting, reduces betrayal)
    PsychologicalSafety,

    /// Ruthless/cutthroat culture (promotes results through internal competition)
    Ruthless,

    /// Bureaucratic culture (slow but stable, procedure-focused)
    Bureaucratic,

    /// Fanatic/zealot culture (fearless, death-embracing)
    Fanatic,

    /// Overwork culture (high productivity but stress and burnout)
    Overwork,

    /// Martyrdom culture (self-sacrifice is honored)
    Martyrdom,

    /// Custom culture tag (game-specific)
    Custom(String),
}

impl CultureTag {
    /// Get a human-readable description of this culture tag
    pub fn description(&self) -> &str {
        match self {
            CultureTag::RiskTaking => "Innovation over safety - failure is tolerated",
            CultureTag::PsychologicalSafety => "Open communication - reporting is rewarded",
            CultureTag::Ruthless => "Results over relationships - survival of the fittest",
            CultureTag::Bureaucratic => "Process over speed - everything by the book",
            CultureTag::Fanatic => "Ideology over life - death holds no fear",
            CultureTag::Overwork => "Productivity over health - rest is weakness",
            CultureTag::Martyrdom => "Sacrifice is honor - dying for the cause is glorious",
            CultureTag::Custom(name) => name,
        }
    }
}

/// Personality traits of individual members
///
/// These represent a member's natural inclinations and temperament.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PersonalityTrait {
    /// Cautious, risk-averse
    Cautious,

    /// Bold, risk-seeking
    Bold,

    /// Competitive, ambitious
    Competitive,

    /// Collaborative, team-oriented
    Collaborative,

    /// Zealous, ideologically driven
    Zealous,

    /// Pragmatic, results-oriented
    Pragmatic,

    /// Workaholic, driven
    Workaholic,

    /// Balanced, moderate
    Balanced,

    /// Custom trait (game-specific)
    Custom(String),
}

/// Member of an organization with cultural alignment
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub id: MemberId,
    pub name: String,

    /// Member's personality traits
    pub personality_traits: HashSet<PersonalityTrait>,

    /// Current stress level (0.0-1.0)
    /// High when culture-personality mismatch exists
    pub stress: f32,

    /// Fervor/devotion to the culture (0.0-1.0)
    /// High when well-aligned with culture
    pub fervor: f32,

    /// Tenure in organization (turns)
    pub tenure: u32,
}

impl Member {
    /// Create a new member
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            personality_traits: HashSet::new(),
            stress: 0.0,
            fervor: 0.5, // Neutral starting point
            tenure: 0,
        }
    }

    /// Add a personality trait
    pub fn with_trait(mut self, trait_: PersonalityTrait) -> Self {
        self.personality_traits.insert(trait_);
        self
    }

    /// Add multiple personality traits
    pub fn with_traits(mut self, traits: Vec<PersonalityTrait>) -> Self {
        self.personality_traits.extend(traits);
        self
    }

    /// Set stress level
    pub fn with_stress(mut self, stress: f32) -> Self {
        self.stress = stress.clamp(0.0, 1.0);
        self
    }

    /// Set fervor level
    pub fn with_fervor(mut self, fervor: f32) -> Self {
        self.fervor = fervor.clamp(0.0, 1.0);
        self
    }

    /// Set tenure
    pub fn with_tenure(mut self, tenure: u32) -> Self {
        self.tenure = tenure;
        self
    }

    /// Check if member has a specific trait
    pub fn has_trait(&self, trait_: &PersonalityTrait) -> bool {
        self.personality_traits.contains(trait_)
    }
}

/// Alignment between member personality and organizational culture
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Alignment {
    /// Perfect match - member thrives in this culture
    Aligned {
        /// Bonus to fervor growth
        fervor_bonus: f32,
    },

    /// Mismatch - member suffers stress
    Misaligned {
        /// Stress accumulation rate
        stress_rate: f32,

        /// Reason for mismatch
        reason: String,
    },

    /// Neutral - no strong reaction
    Neutral,
}

/// Effects that culture has on member behavior
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CultureEffect {
    /// Member acts more boldly (from RiskTaking culture)
    IncreasedRiskTaking { magnitude: f32 },

    /// Member reports more honestly (from PsychologicalSafety)
    ImprovedReporting { magnitude: f32 },

    /// Member becomes more competitive (from Ruthless culture)
    IncreasedCompetition { magnitude: f32 },

    /// Member slows down for procedures (from Bureaucratic culture)
    IncreasedCaution { magnitude: f32 },

    /// Member fearless of death (from Fanatic culture)
    Fearless,

    /// Member works harder but accumulates stress (from Overwork)
    IncreasedProductivity { stress_cost: f32 },

    /// Member willing to sacrifice self (from Martyrdom)
    SelfSacrifice { probability: f32 },

    /// Custom effect (game-specific)
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

/// Errors that can occur in culture operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CultureError {
    FactionNotFound,
    MemberNotFound,
    InvalidCultureTag,
    AlignmentCheckFailed,
}

impl std::fmt::Display for CultureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CultureError::FactionNotFound => write!(f, "Faction not found"),
            CultureError::MemberNotFound => write!(f, "Member not found"),
            CultureError::InvalidCultureTag => write!(f, "Invalid culture tag"),
            CultureError::AlignmentCheckFailed => write!(f, "Alignment check failed"),
        }
    }
}

impl std::error::Error for CultureError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_culture_tag_description() {
        assert_eq!(
            CultureTag::RiskTaking.description(),
            "Innovation over safety - failure is tolerated"
        );
        assert_eq!(
            CultureTag::PsychologicalSafety.description(),
            "Open communication - reporting is rewarded"
        );
    }

    #[test]
    fn test_member_creation() {
        let member = Member::new("m1", "Alice");

        assert_eq!(member.id, "m1");
        assert_eq!(member.name, "Alice");
        assert_eq!(member.stress, 0.0);
        assert_eq!(member.fervor, 0.5);
        assert_eq!(member.tenure, 0);
        assert!(member.personality_traits.is_empty());
    }

    #[test]
    fn test_member_with_trait() {
        let member = Member::new("m1", "Bob")
            .with_trait(PersonalityTrait::Cautious)
            .with_trait(PersonalityTrait::Collaborative);

        assert_eq!(member.personality_traits.len(), 2);
        assert!(member.has_trait(&PersonalityTrait::Cautious));
        assert!(member.has_trait(&PersonalityTrait::Collaborative));
        assert!(!member.has_trait(&PersonalityTrait::Bold));
    }

    #[test]
    fn test_member_with_traits() {
        let member = Member::new("m1", "Charlie")
            .with_traits(vec![PersonalityTrait::Bold, PersonalityTrait::Competitive]);

        assert_eq!(member.personality_traits.len(), 2);
        assert!(member.has_trait(&PersonalityTrait::Bold));
        assert!(member.has_trait(&PersonalityTrait::Competitive));
    }

    #[test]
    fn test_member_stress_clamping() {
        let member = Member::new("m1", "Test").with_stress(1.5); // Should clamp to 1.0

        assert_eq!(member.stress, 1.0);

        let member = Member::new("m2", "Test").with_stress(-0.5); // Should clamp to 0.0

        assert_eq!(member.stress, 0.0);
    }

    #[test]
    fn test_member_fervor_clamping() {
        let member = Member::new("m1", "Test").with_fervor(2.0); // Should clamp to 1.0

        assert_eq!(member.fervor, 1.0);
    }

    #[test]
    fn test_alignment_aligned() {
        let alignment = Alignment::Aligned { fervor_bonus: 0.1 };

        match alignment {
            Alignment::Aligned { fervor_bonus } => assert_eq!(fervor_bonus, 0.1),
            _ => panic!("Expected Aligned"),
        }
    }

    #[test]
    fn test_alignment_misaligned() {
        let alignment = Alignment::Misaligned {
            stress_rate: 0.05,
            reason: "Cautious in RiskTaking culture".to_string(),
        };

        match alignment {
            Alignment::Misaligned {
                stress_rate,
                reason,
            } => {
                assert_eq!(stress_rate, 0.05);
                assert_eq!(reason, "Cautious in RiskTaking culture");
            }
            _ => panic!("Expected Misaligned"),
        }
    }

    #[test]
    fn test_culture_effect_serialization() {
        let effect = CultureEffect::IncreasedRiskTaking { magnitude: 0.3 };

        let json = serde_json::to_string(&effect).unwrap();
        let deserialized: CultureEffect = serde_json::from_str(&json).unwrap();

        assert_eq!(effect, deserialized);
    }

    #[test]
    fn test_culture_error_display() {
        let error = CultureError::FactionNotFound;
        assert_eq!(error.to_string(), "Faction not found");

        let error = CultureError::AlignmentCheckFailed;
        assert_eq!(error.to_string(), "Alignment check failed");
    }

    #[test]
    fn test_custom_culture_tag() {
        let tag = CultureTag::Custom("Pirate Code".to_string());
        assert_eq!(tag.description(), "Pirate Code");
    }

    #[test]
    fn test_member_serialization() {
        let member = Member::new("m1", "Test")
            .with_trait(PersonalityTrait::Bold)
            .with_stress(0.7)
            .with_fervor(0.8);

        let json = serde_json::to_string(&member).unwrap();
        let deserialized: Member = serde_json::from_str(&json).unwrap();

        assert_eq!(member, deserialized);
    }
}
