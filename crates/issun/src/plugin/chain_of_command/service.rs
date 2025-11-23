//! Pure logic service for hierarchy management
//!
//! This service provides stateless functions for promotion eligibility,
//! order compliance calculation, loyalty decay, and authority calculations.

use super::config::ChainOfCommandConfig;
use super::rank_definitions::{AuthorityLevel, RankDefinition};
use super::state::OrganizationHierarchy;
use super::types::{Member, MemberId};

/// Hierarchy service (stateless, pure functions)
///
/// All methods are pure functions with no side effects, making them easy to test.
#[derive(Debug, Clone, Copy, Default)]
pub struct HierarchyService;

impl HierarchyService {
    /// Check if a member is eligible for promotion
    ///
    /// # Promotion Requirements
    ///
    /// 1. Sufficient tenure (>= min_tenure_for_promotion)
    /// 2. Adequate loyalty (>= min_loyalty_for_promotion)
    /// 3. Next rank is consecutive (level + 1)
    ///
    /// # Arguments
    ///
    /// * `member` - The member to check
    /// * `current_rank_def` - Current rank definition
    /// * `next_rank_def` - Target rank definition
    /// * `config` - Configuration with promotion requirements
    ///
    /// # Returns
    ///
    /// `true` if member meets all framework requirements, `false` otherwise
    pub fn can_promote(
        member: &Member,
        current_rank_def: &RankDefinition,
        next_rank_def: &RankDefinition,
        config: &ChainOfCommandConfig,
    ) -> bool {
        // Tenure check
        if member.tenure < config.min_tenure_for_promotion {
            return false;
        }

        // Loyalty check
        if member.loyalty < config.min_loyalty_for_promotion {
            return false;
        }

        // Rank level must be consecutive
        if next_rank_def.level != current_rank_def.level + 1 {
            return false;
        }

        true
    }

    /// Calculate order compliance probability
    ///
    /// # Formula
    ///
    /// `compliance = base_rate * loyalty * morale`
    ///
    /// # Arguments
    ///
    /// * `subordinate` - The member receiving the order
    /// * `superior` - The member issuing the order (unused in current formula)
    /// * `base_rate` - Base compliance rate from config
    ///
    /// # Returns
    ///
    /// Probability (0.0-1.0) that the order will be executed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let subordinate = Member { loyalty: 0.8, morale: 0.7, .. };
    /// let superior = Member { morale: 0.9, .. };
    /// let compliance = HierarchyService::calculate_order_compliance(
    ///     &subordinate,
    ///     &superior,
    ///     0.8,  // 80% base rate
    /// );
    /// // compliance ≈ 0.8 * 0.8 * 0.7 = 0.448 (44.8%)
    /// ```
    pub fn calculate_order_compliance(
        subordinate: &Member,
        _superior: &Member,
        base_rate: f32,
    ) -> f32 {
        // Factors: loyalty and morale
        let loyalty_factor = subordinate.loyalty.clamp(0.0, 1.0);
        let morale_factor = subordinate.morale.clamp(0.0, 1.0);

        // Compliance = base * loyalty * morale
        (base_rate * loyalty_factor * morale_factor).clamp(0.0, 1.0)
    }

    /// Calculate loyalty decay over time (linear decay)
    ///
    /// # Formula
    ///
    /// `new_loyalty = max(0.0, current_loyalty - decay_rate * delta_turns)`
    ///
    /// # Arguments
    ///
    /// * `current_loyalty` - Current loyalty (0.0-1.0)
    /// * `decay_rate` - Decay rate per turn (0.0-1.0)
    /// * `delta_turns` - Number of turns elapsed
    ///
    /// # Returns
    ///
    /// Decayed loyalty value (clamped to 0.0-1.0)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // After 10 turns at 2% decay rate
    /// let decayed = HierarchyService::decay_loyalty(1.0, 0.02, 10);
    /// // decayed = 1.0 - 0.02 * 10 = 0.8
    /// ```
    pub fn decay_loyalty(current_loyalty: f32, decay_rate: f32, delta_turns: u32) -> f32 {
        (current_loyalty - decay_rate * delta_turns as f32).max(0.0)
    }

    /// Calculate loyalty modifier from superior's morale
    ///
    /// # Formula
    ///
    /// `modifier = superior_morale * 0.3`
    ///
    /// Good leaders with high morale boost subordinate loyalty.
    ///
    /// # Arguments
    ///
    /// * `subordinate` - The subordinate (unused in current formula)
    /// * `superior` - The superior whose morale affects loyalty
    ///
    /// # Returns
    ///
    /// Loyalty modifier value (0.0-0.3)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let superior = Member { morale: 0.9, .. };
    /// let modifier = HierarchyService::calculate_loyalty_modifier(
    ///     &subordinate,
    ///     &superior,
    /// );
    /// // modifier = 0.9 * 0.3 = 0.27
    /// ```
    pub fn calculate_loyalty_modifier(_subordinate: &Member, superior: &Member) -> f32 {
        // Good leader boosts subordinate loyalty
        (superior.morale * 0.3).clamp(0.0, 0.3)
    }

    /// Calculate chain depth (distance from supreme commander)
    ///
    /// # Algorithm
    ///
    /// Walks up the hierarchy from member to supreme commander, counting hops.
    ///
    /// # Arguments
    ///
    /// * `member_id` - Starting member
    /// * `hierarchy` - Organization hierarchy
    ///
    /// # Returns
    ///
    /// Number of levels between member and supreme commander
    /// - 0 = supreme commander
    /// - 1 = direct report to supreme commander
    /// - 2 = two levels below supreme commander
    /// - etc.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Commander (depth 0)
    /// //   └─ Captain (depth 1)
    /// //       └─ Sergeant (depth 2)
    /// let depth = HierarchyService::calculate_chain_depth("sergeant", &hierarchy);
    /// // depth = 2
    /// ```
    pub fn calculate_chain_depth(member_id: MemberId, hierarchy: &OrganizationHierarchy) -> u32 {
        let mut depth = 0;
        let mut current = member_id;

        while let Some(member) = hierarchy.get_member(&current) {
            if let Some(superior_id) = &member.superior {
                depth += 1;
                current = superior_id.clone();
            } else {
                break;
            }
        }

        depth
    }

    /// Calculate effective authority (rank authority modified by loyalty)
    ///
    /// # Formula
    ///
    /// `effective_authority = base_authority * loyalty`
    ///
    /// Authority diminishes with low loyalty.
    ///
    /// # Arguments
    ///
    /// * `member` - The member
    /// * `rank_def` - Rank definition
    ///
    /// # Returns
    ///
    /// Effective authority (0.0-1.0)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Captain with 80% loyalty
    /// let captain_rank = RankDefinition { authority_level: AuthorityLevel::Captain, .. };
    /// let member = Member { loyalty: 0.8, .. };
    /// let authority = HierarchyService::calculate_effective_authority(&member, &captain_rank);
    /// // authority = 0.5 * 0.8 = 0.4
    /// ```
    pub fn calculate_effective_authority(member: &Member, rank_def: &RankDefinition) -> f32 {
        let base_authority = match rank_def.authority_level {
            AuthorityLevel::Private => 0.0,
            AuthorityLevel::SquadLeader => 0.25,
            AuthorityLevel::Captain => 0.5,
            AuthorityLevel::Commander => 0.75,
            AuthorityLevel::SupremeCommander => 1.0,
        };

        // Authority diminishes with low loyalty
        (base_authority * member.loyalty).clamp(0.0, 1.0)
    }

    /// Calculate morale impact on order compliance
    ///
    /// # Formula
    ///
    /// Higher morale = higher compliance, but with diminishing returns
    ///
    /// # Arguments
    ///
    /// * `morale` - Current morale (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Compliance multiplier (0.5-1.0)
    pub fn calculate_morale_impact(morale: f32) -> f32 {
        // Even with 0 morale, there's 50% base compliance (fear)
        // At full morale, 100% compliance
        (0.5 + morale * 0.5).clamp(0.5, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::chain_of_command::{Member, OrganizationHierarchy};

    fn create_test_member(id: &str, loyalty: f32, morale: f32, tenure: u32) -> Member {
        Member::new(id, format!("Member {}", id), "private")
            .with_loyalty(loyalty)
            .with_morale(morale)
            .with_tenure(tenure)
    }

    fn create_rank(level: u32, authority: AuthorityLevel) -> RankDefinition {
        RankDefinition::new(
            format!("rank_{}", level),
            format!("Rank {}", level),
            level,
            authority,
        )
    }

    fn create_config() -> ChainOfCommandConfig {
        ChainOfCommandConfig::default()
    }

    #[test]
    fn test_can_promote_with_sufficient_tenure_and_loyalty() {
        let member = create_test_member("m1", 0.8, 0.7, 10);
        let current = create_rank(0, AuthorityLevel::Private);
        let next = create_rank(1, AuthorityLevel::SquadLeader);
        let config = create_config();

        assert!(HierarchyService::can_promote(
            &member, &current, &next, &config
        ));
    }

    #[test]
    fn test_cannot_promote_with_insufficient_tenure() {
        let member = create_test_member("m1", 0.8, 0.7, 2); // tenure = 2, need 5
        let current = create_rank(0, AuthorityLevel::Private);
        let next = create_rank(1, AuthorityLevel::SquadLeader);
        let config = create_config();

        assert!(!HierarchyService::can_promote(
            &member, &current, &next, &config
        ));
    }

    #[test]
    fn test_cannot_promote_with_low_loyalty() {
        let member = create_test_member("m1", 0.3, 0.7, 10); // loyalty = 0.3, need 0.5
        let current = create_rank(0, AuthorityLevel::Private);
        let next = create_rank(1, AuthorityLevel::SquadLeader);
        let config = create_config();

        assert!(!HierarchyService::can_promote(
            &member, &current, &next, &config
        ));
    }

    #[test]
    fn test_cannot_promote_skip_level() {
        let member = create_test_member("m1", 0.8, 0.7, 10);
        let current = create_rank(0, AuthorityLevel::Private);
        let next = create_rank(2, AuthorityLevel::Captain); // Skip level 1
        let config = create_config();

        assert!(!HierarchyService::can_promote(
            &member, &current, &next, &config
        ));
    }

    #[test]
    fn test_calculate_order_compliance() {
        let subordinate = create_test_member("sub", 0.8, 0.7, 5);
        let superior = create_test_member("sup", 1.0, 0.9, 20);

        let compliance = HierarchyService::calculate_order_compliance(
            &subordinate,
            &superior,
            0.8, // 80% base rate
        );

        // 0.8 * 0.8 * 0.7 = 0.448
        assert!((compliance - 0.448).abs() < 0.001);
    }

    #[test]
    fn test_calculate_order_compliance_with_zero_values() {
        let subordinate = create_test_member("sub", 0.0, 0.0, 5);
        let superior = create_test_member("sup", 1.0, 1.0, 20);

        let compliance = HierarchyService::calculate_order_compliance(&subordinate, &superior, 0.8);

        assert_eq!(compliance, 0.0); // No loyalty or morale = no compliance
    }

    #[test]
    fn test_calculate_order_compliance_clamping() {
        let subordinate = create_test_member("sub", 1.5, 1.5, 5); // Values > 1.0
        let superior = create_test_member("sup", 1.0, 1.0, 20);

        let compliance = HierarchyService::calculate_order_compliance(&subordinate, &superior, 0.8);

        assert!(compliance <= 1.0); // Clamped to max 1.0
    }

    #[test]
    fn test_decay_loyalty() {
        let decayed = HierarchyService::decay_loyalty(1.0, 0.02, 10);
        // 1.0 - 0.02 * 10 = 0.8
        assert_eq!(decayed, 0.8);
    }

    #[test]
    fn test_decay_loyalty_to_zero() {
        let decayed = HierarchyService::decay_loyalty(0.1, 0.05, 5);
        // 0.1 - 0.05 * 5 = -0.15 → clamped to 0.0
        assert_eq!(decayed, 0.0);
    }

    #[test]
    fn test_decay_loyalty_no_decay() {
        let decayed = HierarchyService::decay_loyalty(0.8, 0.0, 10);
        assert_eq!(decayed, 0.8); // No decay
    }

    #[test]
    fn test_calculate_loyalty_modifier() {
        let subordinate = create_test_member("sub", 0.5, 0.5, 5);
        let superior = create_test_member("sup", 1.0, 0.9, 20);

        let modifier = HierarchyService::calculate_loyalty_modifier(&subordinate, &superior);
        // 0.9 * 0.3 = 0.27
        assert!((modifier - 0.27).abs() < 0.001);
    }

    #[test]
    fn test_calculate_loyalty_modifier_zero_morale() {
        let subordinate = create_test_member("sub", 0.5, 0.5, 5);
        let superior = create_test_member("sup", 1.0, 0.0, 20);

        let modifier = HierarchyService::calculate_loyalty_modifier(&subordinate, &superior);
        assert_eq!(modifier, 0.0); // No morale = no bonus
    }

    #[test]
    fn test_calculate_chain_depth() {
        let mut hierarchy = OrganizationHierarchy::new();

        // Create chain: commander → captain → sergeant
        let mut commander = create_test_member("commander", 1.0, 1.0, 20);
        commander.superior = None;

        let mut captain = create_test_member("captain", 0.8, 0.8, 10);
        captain.superior = Some("commander".to_string());

        let mut sergeant = create_test_member("sergeant", 0.7, 0.7, 5);
        sergeant.superior = Some("captain".to_string());

        hierarchy.add_member(commander);
        hierarchy.add_member(captain);
        hierarchy.add_member(sergeant);

        assert_eq!(
            HierarchyService::calculate_chain_depth("commander".to_string(), &hierarchy),
            0
        );
        assert_eq!(
            HierarchyService::calculate_chain_depth("captain".to_string(), &hierarchy),
            1
        );
        assert_eq!(
            HierarchyService::calculate_chain_depth("sergeant".to_string(), &hierarchy),
            2
        );
    }

    #[test]
    fn test_calculate_effective_authority() {
        let member = create_test_member("m1", 0.8, 0.7, 10);

        // Private: 0.0 * 0.8 = 0.0
        let private_rank = create_rank(0, AuthorityLevel::Private);
        assert_eq!(
            HierarchyService::calculate_effective_authority(&member, &private_rank),
            0.0
        );

        // Squad Leader: 0.25 * 0.8 = 0.2
        let squad_rank = create_rank(1, AuthorityLevel::SquadLeader);
        assert!(
            (HierarchyService::calculate_effective_authority(&member, &squad_rank) - 0.2).abs()
                < 0.001
        );

        // Captain: 0.5 * 0.8 = 0.4
        let captain_rank = create_rank(2, AuthorityLevel::Captain);
        assert!(
            (HierarchyService::calculate_effective_authority(&member, &captain_rank) - 0.4).abs()
                < 0.001
        );

        // Commander: 0.75 * 0.8 = 0.6
        let commander_rank = create_rank(3, AuthorityLevel::Commander);
        assert!(
            (HierarchyService::calculate_effective_authority(&member, &commander_rank) - 0.6).abs()
                < 0.001
        );

        // Supreme: 1.0 * 0.8 = 0.8
        let supreme_rank = create_rank(4, AuthorityLevel::SupremeCommander);
        assert!(
            (HierarchyService::calculate_effective_authority(&member, &supreme_rank) - 0.8).abs()
                < 0.001
        );
    }

    #[test]
    fn test_calculate_effective_authority_zero_loyalty() {
        let member = create_test_member("m1", 0.0, 0.7, 10);
        let captain_rank = create_rank(2, AuthorityLevel::Captain);

        assert_eq!(
            HierarchyService::calculate_effective_authority(&member, &captain_rank),
            0.0
        );
    }

    #[test]
    fn test_calculate_morale_impact() {
        assert_eq!(HierarchyService::calculate_morale_impact(0.0), 0.5); // Minimum
        assert_eq!(HierarchyService::calculate_morale_impact(0.5), 0.75);
        assert_eq!(HierarchyService::calculate_morale_impact(1.0), 1.0); // Maximum
    }

    #[test]
    fn test_calculate_morale_impact_clamping() {
        assert_eq!(HierarchyService::calculate_morale_impact(-0.5), 0.5); // Clamp to min
        assert_eq!(HierarchyService::calculate_morale_impact(1.5), 1.0); // Clamp to max
    }
}
