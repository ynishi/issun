//! Configuration for holacracy plugin
//!
//! Defines tunable parameters for task assignment, bidding, and self-organization.

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for holacracy plugin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolacracyConfig {
    /// Task assignment mode
    pub assignment_mode: TaskAssignmentMode,

    /// Bidding configuration
    pub bidding: BiddingConfig,

    /// Maximum number of tasks a member can have assigned
    pub max_tasks_per_member: usize,

    /// Maximum number of roles a member can fill
    pub max_roles_per_member: usize,

    /// Task priority boost for critical tasks (multiplier)
    pub critical_priority_boost: f32,

    /// Skill match weight in bid scoring (0.0-1.0)
    pub skill_match_weight: f32,

    /// Workload weight in bid scoring (0.0-1.0)
    pub workload_weight: f32,

    /// Interest weight in bid scoring (0.0-1.0)
    pub interest_weight: f32,

    /// Enable dynamic role switching
    pub enable_role_switching: bool,

    /// Role switching cooldown (in turns)
    pub role_switch_cooldown: u64,

    /// Minimum skill level to bid on operational roles (0.0-1.0)
    pub min_skill_level_for_bid: f32,

    /// Maximum circle nesting depth
    pub max_circle_depth: usize,
}

impl HolacracyConfig {
    /// Create a new config with validation
    pub fn new() -> Result<Self, String> {
        let config = Self::default();
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        // Validate weights
        let total_weight =
            self.skill_match_weight + self.workload_weight + self.interest_weight;
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(format!(
                "Bid scoring weights must sum to 1.0, got {}",
                total_weight
            ));
        }

        if self.skill_match_weight < 0.0
            || self.skill_match_weight > 1.0
            || self.workload_weight < 0.0
            || self.workload_weight > 1.0
            || self.interest_weight < 0.0
            || self.interest_weight > 1.0
        {
            return Err("All bid weights must be in range 0.0-1.0".to_string());
        }

        // Validate min skill level
        if self.min_skill_level_for_bid < 0.0 || self.min_skill_level_for_bid > 1.0 {
            return Err("min_skill_level_for_bid must be in range 0.0-1.0".to_string());
        }

        // Validate bidding config
        self.bidding.validate()?;

        // Validate limits
        if self.max_tasks_per_member == 0 {
            return Err("max_tasks_per_member must be at least 1".to_string());
        }

        if self.max_roles_per_member == 0 {
            return Err("max_roles_per_member must be at least 1".to_string());
        }

        if self.max_circle_depth == 0 {
            return Err("max_circle_depth must be at least 1".to_string());
        }

        Ok(())
    }

    /// Builder: Set assignment mode
    pub fn with_assignment_mode(mut self, mode: TaskAssignmentMode) -> Self {
        self.assignment_mode = mode;
        self
    }

    /// Builder: Set bidding config
    pub fn with_bidding_config(mut self, config: BiddingConfig) -> Self {
        self.bidding = config;
        self
    }

    /// Builder: Set max tasks per member
    pub fn with_max_tasks(mut self, max: usize) -> Self {
        self.max_tasks_per_member = max;
        self
    }

    /// Builder: Set max roles per member
    pub fn with_max_roles(mut self, max: usize) -> Self {
        self.max_roles_per_member = max;
        self
    }

    /// Builder: Set skill weights
    pub fn with_skill_weights(mut self, skill: f32, workload: f32, interest: f32) -> Self {
        self.skill_match_weight = skill;
        self.workload_weight = workload;
        self.interest_weight = interest;
        self
    }

    /// Builder: Enable/disable role switching
    pub fn with_role_switching(mut self, enabled: bool) -> Self {
        self.enable_role_switching = enabled;
        self
    }

    /// Builder: Set role switching cooldown
    pub fn with_role_switch_cooldown(mut self, cooldown: u64) -> Self {
        self.role_switch_cooldown = cooldown;
        self
    }

    /// Builder: Set minimum skill level for bidding
    pub fn with_min_skill_level(mut self, level: f32) -> Self {
        self.min_skill_level_for_bid = level;
        self
    }

    /// Builder: Set maximum circle depth
    pub fn with_max_circle_depth(mut self, depth: usize) -> Self {
        self.max_circle_depth = depth;
        self
    }
}

impl Default for HolacracyConfig {
    fn default() -> Self {
        Self {
            assignment_mode: TaskAssignmentMode::SemiAutonomous,
            bidding: BiddingConfig::default(),
            max_tasks_per_member: 5,
            max_roles_per_member: 3,
            critical_priority_boost: 2.0,
            skill_match_weight: 0.5,
            workload_weight: 0.3,
            interest_weight: 0.2,
            enable_role_switching: true,
            role_switch_cooldown: 5,
            min_skill_level_for_bid: 0.3,
            max_circle_depth: 5,
        }
    }
}

/// Mode for task assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskAssignmentMode {
    /// Fully autonomous bidding - tasks auto-assigned to best bid
    FullyAutonomous,
    /// Semi-autonomous - best bids presented, requires approval
    SemiAutonomous,
    /// Manual - all assignments require explicit approval
    Manual,
}

impl Default for TaskAssignmentMode {
    fn default() -> Self {
        TaskAssignmentMode::SemiAutonomous
    }
}

/// Configuration for bidding system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiddingConfig {
    /// Duration of bidding period (in turns)
    pub bidding_duration: u64,

    /// Minimum number of bids required before auto-assignment
    pub min_bids_required: usize,

    /// Enable bid retractions (members can withdraw bids)
    pub allow_bid_retraction: bool,

    /// Penalty for bid retraction (reduces future bid scores)
    pub retraction_penalty: f32,

    /// Enable overbidding (members can bid on more than max_tasks)
    pub allow_overbidding: bool,

    /// Overbid penalty multiplier (reduces bid score if overbidding)
    pub overbid_penalty_multiplier: f32,
}

impl BiddingConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.bidding_duration == 0 {
            return Err("bidding_duration must be at least 1".to_string());
        }

        if self.retraction_penalty < 0.0 || self.retraction_penalty > 1.0 {
            return Err("retraction_penalty must be in range 0.0-1.0".to_string());
        }

        if self.overbid_penalty_multiplier < 0.0 {
            return Err("overbid_penalty_multiplier must be non-negative".to_string());
        }

        Ok(())
    }

    /// Builder: Set bidding duration
    pub fn with_duration(mut self, duration: u64) -> Self {
        self.bidding_duration = duration;
        self
    }

    /// Builder: Set minimum bids required
    pub fn with_min_bids(mut self, min: usize) -> Self {
        self.min_bids_required = min;
        self
    }

    /// Builder: Enable/disable bid retraction
    pub fn with_retraction(mut self, allowed: bool, penalty: f32) -> Self {
        self.allow_bid_retraction = allowed;
        self.retraction_penalty = penalty;
        self
    }

    /// Builder: Enable/disable overbidding
    pub fn with_overbidding(mut self, allowed: bool, penalty: f32) -> Self {
        self.allow_overbidding = allowed;
        self.overbid_penalty_multiplier = penalty;
        self
    }
}

impl Default for BiddingConfig {
    fn default() -> Self {
        Self {
            bidding_duration: 3,
            min_bids_required: 1,
            allow_bid_retraction: true,
            retraction_penalty: 0.1,
            allow_overbidding: false,
            overbid_penalty_multiplier: 0.5,
        }
    }
}

// Implement Resource trait for HolacracyConfig
impl Resource for HolacracyConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = HolacracyConfig::default();
        assert_eq!(config.assignment_mode, TaskAssignmentMode::SemiAutonomous);
        assert_eq!(config.max_tasks_per_member, 5);
        assert_eq!(config.max_roles_per_member, 3);
    }

    #[test]
    fn test_config_validation_success() {
        let config = HolacracyConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_weights_sum() {
        let mut config = HolacracyConfig::default();
        config.skill_match_weight = 0.5;
        config.workload_weight = 0.3;
        config.interest_weight = 0.1; // Sum = 0.9, should fail

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_weight_range() {
        let mut config = HolacracyConfig::default();
        config.skill_match_weight = 1.5; // Out of range

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_min_skill_level() {
        let mut config = HolacracyConfig::default();
        config.min_skill_level_for_bid = 1.5; // Out of range

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = HolacracyConfig::default()
            .with_assignment_mode(TaskAssignmentMode::FullyAutonomous)
            .with_max_tasks(10)
            .with_max_roles(5)
            .with_role_switching(false);

        assert_eq!(config.assignment_mode, TaskAssignmentMode::FullyAutonomous);
        assert_eq!(config.max_tasks_per_member, 10);
        assert_eq!(config.max_roles_per_member, 5);
        assert!(!config.enable_role_switching);
    }

    #[test]
    fn test_config_builder_with_skill_weights() {
        let config = HolacracyConfig::default().with_skill_weights(0.6, 0.3, 0.1);

        assert_eq!(config.skill_match_weight, 0.6);
        assert_eq!(config.workload_weight, 0.3);
        assert_eq!(config.interest_weight, 0.1);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_bidding_config_default() {
        let config = BiddingConfig::default();
        assert_eq!(config.bidding_duration, 3);
        assert_eq!(config.min_bids_required, 1);
        assert!(config.allow_bid_retraction);
    }

    #[test]
    fn test_bidding_config_validation_success() {
        let config = BiddingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_bidding_config_validation_duration() {
        let mut config = BiddingConfig::default();
        config.bidding_duration = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_bidding_config_validation_penalty_range() {
        let mut config = BiddingConfig::default();
        config.retraction_penalty = 1.5;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_bidding_config_builder() {
        let config = BiddingConfig::default()
            .with_duration(5)
            .with_min_bids(2)
            .with_retraction(false, 0.0)
            .with_overbidding(true, 0.3);

        assert_eq!(config.bidding_duration, 5);
        assert_eq!(config.min_bids_required, 2);
        assert!(!config.allow_bid_retraction);
        assert!(config.allow_overbidding);
        assert_eq!(config.overbid_penalty_multiplier, 0.3);
    }

    #[test]
    fn test_assignment_mode_default() {
        assert_eq!(
            TaskAssignmentMode::default(),
            TaskAssignmentMode::SemiAutonomous
        );
    }

    #[test]
    fn test_config_new() {
        let result = HolacracyConfig::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_max_tasks() {
        let mut config = HolacracyConfig::default();
        config.max_tasks_per_member = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_max_roles() {
        let mut config = HolacracyConfig::default();
        config.max_roles_per_member = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_max_circle_depth() {
        let mut config = HolacracyConfig::default();
        config.max_circle_depth = 0;

        assert!(config.validate().is_err());
    }
}
