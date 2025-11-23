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
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn target_archetype(&self) -> OrgArchetype {
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
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn target_archetype(&self) -> OrgArchetype {
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
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn target_archetype(&self) -> OrgArchetype {
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

/// Default converter: Holacracy → Social
///
/// Direct transformation from self-organization to network without hierarchy phase.
///
/// # Data Mapping
///
/// - Task collaboration history → Trust relationships
/// - Circle memberships → Network clusters
/// - Skill proficiency → Social capital
pub struct HolacracyToSocialConverter;

impl OrgConverter for HolacracyToSocialConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "social",
            "converted_from": "holacracy",
            "network_edges": source_data.get("collaborations").unwrap_or(&json!([])),
            "clusters": source_data.get("circles").unwrap_or(&json!([])),
            "social_capital": source_data.get("skill_scores").unwrap_or(&json!({})),
            "note": "Converted from task-based collaboration to social influence network"
        });

        Ok(converted)
    }
}

/// Default converter: Holacracy → Culture
///
/// Rapid radicalization from self-organizing collective to cult.
///
/// # Data Mapping
///
/// - Shared purpose → Dogma
/// - High-performing members → Zealots
/// - Circle rituals → Cultural practices
pub struct HolacracyToCultureConverter;

impl OrgConverter for HolacracyToCultureConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Culture
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "culture",
            "converted_from": "holacracy",
            "dogmas": source_data.get("shared_purpose").unwrap_or(&json!([])),
            "zealots": source_data.get("high_performers").unwrap_or(&json!([])),
            "rituals": source_data.get("circle_practices").unwrap_or(&json!([])),
            "note": "Converted from self-organizing purpose to cultural zealotry"
        });

        Ok(converted)
    }
}

/// Default converter: Hierarchy → Holacracy
///
/// Organizational reform from bureaucracy to self-organization.
///
/// # Data Mapping
///
/// - Departments → Circles
/// - Positions → Roles
/// - Command queue → Task pool
/// - Middle management eliminated
pub struct HierarchyToHolacracyConverter;

impl OrgConverter for HierarchyToHolacracyConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "holacracy",
            "converted_from": "hierarchy",
            "circles": source_data.get("departments").unwrap_or(&json!([])),
            "roles": source_data.get("positions").unwrap_or(&json!([])),
            "task_pool": source_data.get("command_queue").unwrap_or(&json!([])),
            "note": "Converted from hierarchical departments to self-organizing circles"
        });

        Ok(converted)
    }
}

/// Default converter: Hierarchy → Culture
///
/// Authoritarian regime becomes personality cult.
///
/// # Data Mapping
///
/// - Leader → Divine figure
/// - Orders → Sacred commandments
/// - Loyalty → Fanatical devotion
pub struct HierarchyToCultureConverter;

impl OrgConverter for HierarchyToCultureConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Culture
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "culture",
            "converted_from": "hierarchy",
            "divine_leader": source_data.get("top_leader").unwrap_or(&json!(null)),
            "commandments": source_data.get("orders").unwrap_or(&json!([])),
            "devotion_level": source_data.get("loyalty_average").unwrap_or(&json!(0.0)),
            "note": "Converted from hierarchical authority to personality cult"
        });

        Ok(converted)
    }
}

/// Default converter: Social → Holacracy
///
/// Network coalesces into purposeful self-organization.
///
/// # Data Mapping
///
/// - Influential members → Circle leads
/// - Network clusters → Circles
/// - Shared goals → Task definitions
pub struct SocialToHolacracyConverter;

impl OrgConverter for SocialToHolacracyConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "holacracy",
            "converted_from": "social",
            "circles": source_data.get("clusters").unwrap_or(&json!([])),
            "circle_leads": source_data.get("influential_members").unwrap_or(&json!([])),
            "tasks": source_data.get("shared_goals").unwrap_or(&json!([])),
            "note": "Converted from social network to purposeful self-organization"
        });

        Ok(converted)
    }
}

/// Default converter: Social → Hierarchy
///
/// Informal network formalizes into hierarchy.
///
/// # Data Mapping
///
/// - High-influence members → Leaders
/// - Network structure → Reporting hierarchy
/// - Favors owed → Formal obligations
pub struct SocialToHierarchyConverter;

impl OrgConverter for SocialToHierarchyConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "hierarchy",
            "converted_from": "social",
            "leaders": source_data.get("high_influence").unwrap_or(&json!([])),
            "reporting_structure": source_data.get("network_edges").unwrap_or(&json!([])),
            "formal_obligations": source_data.get("favors_owed").unwrap_or(&json!([])),
            "note": "Converted from informal influence network to formal hierarchy"
        });

        Ok(converted)
    }
}

/// Default converter: Culture → Holacracy
///
/// Cult deprogramming into rational self-organization.
///
/// # Data Mapping
///
/// - Dogmas → Shared principles
/// - Rituals → Work practices
/// - Former zealots → Team members
pub struct CultureToHolacracyConverter;

impl OrgConverter for CultureToHolacracyConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Culture
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Holacracy
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "holacracy",
            "converted_from": "culture",
            "principles": source_data.get("dogmas").unwrap_or(&json!([])),
            "practices": source_data.get("rituals").unwrap_or(&json!([])),
            "members": source_data.get("zealots").unwrap_or(&json!([])),
            "note": "Converted from cultural dogma to rational self-organization"
        });

        Ok(converted)
    }
}

/// Default converter: Culture → Hierarchy
///
/// Cult institutionalizes into formal structure.
///
/// # Data Mapping
///
/// - Charismatic leader → Formal authority
/// - Inner circle → Executive committee
/// - Dogmas → Official policies
pub struct CultureToHierarchyConverter;

impl OrgConverter for CultureToHierarchyConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Culture
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Hierarchy
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "hierarchy",
            "converted_from": "culture",
            "formal_leader": source_data.get("charismatic_leader").unwrap_or(&json!(null)),
            "executive_committee": source_data.get("inner_circle").unwrap_or(&json!([])),
            "policies": source_data.get("dogmas").unwrap_or(&json!([])),
            "note": "Converted from cultural movement to institutional hierarchy"
        });

        Ok(converted)
    }
}

/// Default converter: Culture → Social
///
/// Cult dissolution into informal networks.
///
/// # Data Mapping
///
/// - Former members → Network nodes
/// - Shared beliefs → Network bonds
/// - Charisma → Social influence
pub struct CultureToSocialConverter;

impl OrgConverter for CultureToSocialConverter {
    fn source_archetype(&self) -> OrgArchetype {
        OrgArchetype::Culture
    }

    fn target_archetype(&self) -> OrgArchetype {
        OrgArchetype::Social
    }

    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError> {
        let converted = json!({
            "archetype": "social",
            "converted_from": "culture",
            "network_nodes": source_data.get("members").unwrap_or(&json!([])),
            "bonds": source_data.get("shared_beliefs").unwrap_or(&json!([])),
            "influence_map": source_data.get("charisma_scores").unwrap_or(&json!({})),
            "note": "Converted from cultural cult to informal social network"
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
        assert_eq!(converter.source_archetype(), OrgArchetype::Holacracy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Hierarchy);

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
        assert_eq!(converter.source_archetype(), OrgArchetype::Hierarchy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Social);

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
        assert_eq!(converter.source_archetype(), OrgArchetype::Social);
        assert_eq!(converter.target_archetype(), OrgArchetype::Culture);

        let source = json!({
            "factions": ["faction_a", "faction_b"],
            "social_capital": 0.9
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "culture");
        assert_eq!(result["fervor"], 0.9);
    }

    #[test]
    fn test_holacracy_to_social_converter() {
        let converter = HolacracyToSocialConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Holacracy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Social);

        let source = json!({
            "collaborations": [{"a": "b"}],
            "circles": ["circle1"],
            "skill_scores": {"member1": 0.9}
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "social");
        assert_eq!(result["converted_from"], "holacracy");
    }

    #[test]
    fn test_holacracy_to_culture_converter() {
        let converter = HolacracyToCultureConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Holacracy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Culture);

        let source = json!({
            "shared_purpose": ["mission1"],
            "high_performers": ["member1", "member2"],
            "circle_practices": ["daily_standup"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "culture");
        assert_eq!(result["converted_from"], "holacracy");
    }

    #[test]
    fn test_hierarchy_to_holacracy_converter() {
        let converter = HierarchyToHolacracyConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Hierarchy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Holacracy);

        let source = json!({
            "departments": ["engineering", "sales"],
            "positions": ["dev", "manager"],
            "command_queue": ["task1"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "holacracy");
        assert_eq!(result["converted_from"], "hierarchy");
    }

    #[test]
    fn test_hierarchy_to_culture_converter() {
        let converter = HierarchyToCultureConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Hierarchy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Culture);

        let source = json!({
            "top_leader": "supreme_leader",
            "orders": ["order1", "order2"],
            "loyalty_average": 0.95
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "culture");
        assert_eq!(result["converted_from"], "hierarchy");
        assert_eq!(result["devotion_level"], 0.95);
    }

    #[test]
    fn test_social_to_holacracy_converter() {
        let converter = SocialToHolacracyConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Social);
        assert_eq!(converter.target_archetype(), OrgArchetype::Holacracy);

        let source = json!({
            "clusters": ["cluster1", "cluster2"],
            "influential_members": ["leader1"],
            "shared_goals": ["goal1"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "holacracy");
        assert_eq!(result["converted_from"], "social");
    }

    #[test]
    fn test_social_to_hierarchy_converter() {
        let converter = SocialToHierarchyConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Social);
        assert_eq!(converter.target_archetype(), OrgArchetype::Hierarchy);

        let source = json!({
            "high_influence": ["influencer1"],
            "network_edges": [{"from": "A", "to": "B"}],
            "favors_owed": ["favor1"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "hierarchy");
        assert_eq!(result["converted_from"], "social");
    }

    #[test]
    fn test_culture_to_holacracy_converter() {
        let converter = CultureToHolacracyConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Culture);
        assert_eq!(converter.target_archetype(), OrgArchetype::Holacracy);

        let source = json!({
            "dogmas": ["belief1", "belief2"],
            "rituals": ["ritual1"],
            "zealots": ["member1", "member2"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "holacracy");
        assert_eq!(result["converted_from"], "culture");
    }

    #[test]
    fn test_culture_to_hierarchy_converter() {
        let converter = CultureToHierarchyConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Culture);
        assert_eq!(converter.target_archetype(), OrgArchetype::Hierarchy);

        let source = json!({
            "charismatic_leader": "leader1",
            "inner_circle": ["member1", "member2"],
            "dogmas": ["dogma1"]
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "hierarchy");
        assert_eq!(result["converted_from"], "culture");
    }

    #[test]
    fn test_culture_to_social_converter() {
        let converter = CultureToSocialConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Culture);
        assert_eq!(converter.target_archetype(), OrgArchetype::Social);

        let source = json!({
            "members": ["member1", "member2"],
            "shared_beliefs": ["belief1"],
            "charisma_scores": {"member1": 0.8}
        });

        let result = converter.convert(&source).unwrap();
        assert_eq!(result["archetype"], "social");
        assert_eq!(result["converted_from"], "culture");
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
