//! Default converter and condition implementations
//!
//! Provides example implementations that games can use as-is or as reference
//! for creating their own custom converters and conditions.

use super::super::types::{OrgArchetype, OrgSuiteError, TransitionTrigger};
use super::condition::{ConditionContext, TransitionCondition};
use super::converter::OrgConverter;
use serde_json::json;

// ========== Default Converters ==========

/// Default converter: Holacracy → Hierarchy
///
/// Maps self-organizing circles to hierarchical structure when scaling up.
///
/// # Data Mapping
///
/// - Task pool → Command queue
/// - Circles → Departments
/// - Role assignments → Position assignments
/// - High-activity members → Department leaders
pub struct HolacracyToHierarchyConverter;

impl OrgConverter for HolacracyToHierarchyConverter {
    fn from_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn to_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        // Example conversion logic
        // In a real implementation, this would extract actual Holacracy data
        // and transform it to Hierarchy format

        let converted = json!({
            "archetype": "hierarchy",
            "converted_from": "holacracy",
            "departments": source_data.get("circles").unwrap_or(&json!([])),
            "command_queue": source_data.get("task_pool").unwrap_or(&json!([])),
            "positions": source_data.get("roles").unwrap_or(&json!([])),
            "note": "Converted from self-organizing circles to hierarchical departments"
        });

        Ok(converted)
    }
}

/// Default converter: Hierarchy → Social
///
/// Maps hierarchical authority to social network when authority breaks down.
///
/// # Data Mapping
///
/// - Superior-subordinate relationships → Trust network edges
/// - Rank → Social influence score
/// - Tax rate → Bribery cost
/// - Loyalty → Trust strength
pub struct HierarchyToSocialConverter;

impl OrgConverter for HierarchyToSocialConverter {
    fn from_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn to_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "social",
            "converted_from": "hierarchy",
            "network_edges": source_data.get("relationships").unwrap_or(&json!([])),
            "influence_scores": source_data.get("ranks").unwrap_or(&json!({})),
            "bribery_costs": source_data.get("tax_rate").unwrap_or(&json!(0.0)),
            "note": "Converted from hierarchical command to social influence network"
        });

        Ok(converted)
    }
}

/// Default converter: Social → Culture
///
/// Maps social networks to cultural memes when fervor rises.
///
/// # Data Mapping
///
/// - Network centrality → Charismatic leaders
/// - Factions → Dogma tags
/// - Social capital → Fervor level
/// - Shared secrets → Sacred knowledge
pub struct SocialToCultureConverter;

impl OrgConverter for SocialToCultureConverter {
    fn from_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn to_archetype(&self) -> OrgArchetype {
        OrgArchetype::Culture
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "culture",
            "converted_from": "social",
            "dogmas": source_data.get("factions").unwrap_or(&json!([])),
            "fervor": source_data.get("social_capital").unwrap_or(&json!(0.0)),
            "charismatic_leaders": source_data.get("high_centrality_members").unwrap_or(&json!([])),
            "note": "Converted from social network to cultural meme organization"
        });

        Ok(converted)
    }
}

// ========== Default Conditions ==========

/// Scaling condition: Triggers when member count exceeds threshold
///
/// Typically used for Holacracy → Hierarchy transition when organization
/// grows too large for self-organization.
#[derive(Debug, Clone)]
pub struct ScalingCondition {
    /// Member count threshold
    pub threshold: usize,
    /// Source archetype to check
    pub from_archetype: OrgArchetype,
    /// Target archetype if triggered
    pub to_archetype: OrgArchetype,
}

impl ScalingCondition {
    /// Create a new scaling condition
    ///
    /// # Example
    ///
    /// ```ignore
    /// let condition = ScalingCondition::new(50, OrgArchetype::Holacracy, OrgArchetype::Hierarchy);
    /// ```
    pub fn new(threshold: usize, from: OrgArchetype, to: OrgArchetype) -> Self {
        Self {
            threshold,
            from_archetype: from,
            to_archetype: to,
        }
    }
}

impl TransitionCondition for ScalingCondition {
    fn evaluate(
        &self,
        _faction_id: &str,
        current: OrgArchetype,
        context: &ConditionContext,
    ) -> Option<TransitionTrigger> {
        if current == self.from_archetype && context.member_count >= self.threshold {
            Some(TransitionTrigger::Scaling {
                from: self.from_archetype,
                to: self.to_archetype,
                member_count: context.member_count,
            })
        } else {
            None
        }
    }
}

/// Decay condition: Triggers when corruption exceeds threshold
///
/// Typically used for Hierarchy → Social transition when authority
/// breaks down due to corruption.
#[derive(Debug, Clone)]
pub struct DecayCondition {
    /// Corruption threshold (0.0-1.0)
    pub threshold: f32,
    /// Source archetype to check
    pub from_archetype: OrgArchetype,
    /// Target archetype if triggered
    pub to_archetype: OrgArchetype,
}

impl DecayCondition {
    /// Create a new decay condition
    pub fn new(threshold: f32, from: OrgArchetype, to: OrgArchetype) -> Self {
        Self {
            threshold,
            from_archetype: from,
            to_archetype: to,
        }
    }
}

impl TransitionCondition for DecayCondition {
    fn evaluate(
        &self,
        _faction_id: &str,
        current: OrgArchetype,
        context: &ConditionContext,
    ) -> Option<TransitionTrigger> {
        if current == self.from_archetype && context.corruption_level >= self.threshold {
            Some(TransitionTrigger::Decay {
                from: self.from_archetype,
                to: self.to_archetype,
                corruption_level: context.corruption_level,
            })
        } else {
            None
        }
    }
}

/// Radicalization condition: Triggers when fervor exceeds threshold
///
/// Typically used for Social → Culture transition when fervor
/// creates cult-like behavior.
#[derive(Debug, Clone)]
pub struct RadicalizationCondition {
    /// Fervor threshold (0.0-1.0)
    pub threshold: f32,
    /// Source archetype to check
    pub from_archetype: OrgArchetype,
    /// Target archetype if triggered
    pub to_archetype: OrgArchetype,
}

impl RadicalizationCondition {
    /// Create a new radicalization condition
    pub fn new(threshold: f32, from: OrgArchetype, to: OrgArchetype) -> Self {
        Self {
            threshold,
            from_archetype: from,
            to_archetype: to,
        }
    }
}

impl TransitionCondition for RadicalizationCondition {
    fn evaluate(
        &self,
        _faction_id: &str,
        current: OrgArchetype,
        context: &ConditionContext,
    ) -> Option<TransitionTrigger> {
        if current == self.from_archetype && context.fervor_level >= self.threshold {
            Some(TransitionTrigger::Radicalization {
                from: self.from_archetype,
                to: self.to_archetype,
                fervor_level: context.fervor_level,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Converter Tests ==========

    #[test]
    fn test_holacracy_to_hierarchy_converter() {
        let converter = HolacracyToHierarchyConverter;
        assert_eq!(converter.from_archetype(), OrgArchetype::Holacracy);
        assert_eq!(converter.to_archetype(), OrgArchetype::Hierarchy);

        let source = json!({
            "circles": ["engineering", "sales"],
            "task_pool": ["task1", "task2"],
            "roles": ["developer", "manager"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "hierarchy");
        assert_eq!(result["converted_from"], "holacracy");
    }

    #[test]
    fn test_hierarchy_to_social_converter() {
        let converter = HierarchyToSocialConverter;
        assert_eq!(converter.from_archetype(), OrgArchetype::Hierarchy);
        assert_eq!(converter.to_archetype(), OrgArchetype::Social);

        let source = json!({
            "relationships": [{"from": "A", "to": "B"}],
            "ranks": {"A": 5, "B": 3}
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "social");
    }

    #[test]
    fn test_social_to_culture_converter() {
        let converter = SocialToCultureConverter;
        assert_eq!(converter.from_archetype(), OrgArchetype::Social);
        assert_eq!(converter.to_archetype(), OrgArchetype::Culture);

        let source = json!({
            "factions": ["faction_a", "faction_b"],
            "social_capital": 0.9
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "culture");
        assert_eq!(result["fervor"], 0.9);
    }

    // ========== Condition Tests ==========

    #[test]
    fn test_scaling_condition_triggered() {
        let condition = ScalingCondition::new(50, OrgArchetype::Holacracy, OrgArchetype::Hierarchy);

        let context = ConditionContext {
            member_count: 60,
            ..Default::default()
        };

        let result = condition.evaluate("test", OrgArchetype::Holacracy, &context);
        assert!(result.is_some());

        match result.unwrap() {
            TransitionTrigger::Scaling { member_count, .. } => {
                assert_eq!(member_count, 60);
            }
            _ => panic!("Expected Scaling trigger"),
        }
    }

    #[test]
    fn test_scaling_condition_not_triggered() {
        let condition = ScalingCondition::new(50, OrgArchetype::Holacracy, OrgArchetype::Hierarchy);

        let context = ConditionContext {
            member_count: 30,
            ..Default::default()
        };

        let result = condition.evaluate("test", OrgArchetype::Holacracy, &context);
        assert!(result.is_none());
    }

    #[test]
    fn test_decay_condition_triggered() {
        let condition = DecayCondition::new(0.8, OrgArchetype::Hierarchy, OrgArchetype::Social);

        let context = ConditionContext {
            corruption_level: 0.9,
            ..Default::default()
        };

        let result = condition.evaluate("test", OrgArchetype::Hierarchy, &context);
        assert!(result.is_some());
    }

    #[test]
    fn test_radicalization_condition_triggered() {
        let condition =
            RadicalizationCondition::new(0.9, OrgArchetype::Social, OrgArchetype::Culture);

        let context = ConditionContext {
            fervor_level: 0.95,
            ..Default::default()
        };

        let result = condition.evaluate("test", OrgArchetype::Social, &context);
        assert!(result.is_some());

        match result.unwrap() {
            TransitionTrigger::Radicalization { fervor_level, .. } => {
                assert_eq!(fervor_level, 0.95);
            }
            _ => panic!("Expected Radicalization trigger"),
        }
    }

    #[test]
    fn test_conditions_wrong_archetype() {
        let scaling = ScalingCondition::new(50, OrgArchetype::Holacracy, OrgArchetype::Hierarchy);
        let decay = DecayCondition::new(0.8, OrgArchetype::Hierarchy, OrgArchetype::Social);

        let context = ConditionContext {
            member_count: 100,
            corruption_level: 0.9,
            ..Default::default()
        };

        // Scaling condition on Hierarchy (wrong archetype)
        assert!(scaling
            .evaluate("test", OrgArchetype::Hierarchy, &context)
            .is_none());

        // Decay condition on Holacracy (wrong archetype)
        assert!(decay
            .evaluate("test", OrgArchetype::Holacracy, &context)
            .is_none());
    }
}
