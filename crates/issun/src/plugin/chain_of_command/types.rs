//! Core data types for ChainOfCommandPlugin

use serde::{Deserialize, Serialize};

/// Unique identifier for a member
pub type MemberId = String;

/// Unique identifier for a rank
pub type RankId = String;

/// Unique identifier for a faction
pub type FactionId = String;

/// Member of an organization
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub id: MemberId,
    pub name: String,

    /// Current rank
    pub rank: RankId,

    /// Direct superior (None for supreme commander)
    pub superior: Option<MemberId>,

    /// Loyalty to organization (0.0-1.0)
    pub loyalty: f32,

    /// Current morale (0.0-1.0)
    pub morale: f32,

    /// Total tenure in organization (turns)
    pub tenure: u32,

    /// Turns since last promotion
    pub turns_since_promotion: u32,
}

impl Member {
    /// Create a new member
    pub fn new(id: impl Into<String>, name: impl Into<String>, rank: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rank: rank.into(),
            superior: None,
            loyalty: 1.0,
            morale: 1.0,
            tenure: 0,
            turns_since_promotion: 0,
        }
    }

    /// Set superior
    pub fn with_superior(mut self, superior_id: impl Into<String>) -> Self {
        self.superior = Some(superior_id.into());
        self
    }

    /// Set loyalty
    pub fn with_loyalty(mut self, loyalty: f32) -> Self {
        self.loyalty = loyalty.clamp(0.0, 1.0);
        self
    }

    /// Set morale
    pub fn with_morale(mut self, morale: f32) -> Self {
        self.morale = morale.clamp(0.0, 1.0);
        self
    }

    /// Set tenure
    pub fn with_tenure(mut self, tenure: u32) -> Self {
        self.tenure = tenure;
        self
    }
}

/// Order issued through chain of command
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub order_type: OrderType,
    pub priority: Priority,
}

impl Order {
    /// Create a new order
    pub fn new(order_type: OrderType, priority: Priority) -> Self {
        Self {
            order_type,
            priority,
        }
    }

    /// Create an attack order
    pub fn attack(target: impl Into<String>) -> Self {
        Self::new(
            OrderType::Attack {
                target: target.into(),
            },
            Priority::Normal,
        )
    }

    /// Create a defend order
    pub fn defend(location: impl Into<String>) -> Self {
        Self::new(
            OrderType::Defend {
                location: location.into(),
            },
            Priority::Normal,
        )
    }

    /// Create a move order
    pub fn move_to(destination: impl Into<String>) -> Self {
        Self::new(
            OrderType::Move {
                destination: destination.into(),
            },
            Priority::Normal,
        )
    }

    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

/// Types of orders that can be issued
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    /// Attack target
    Attack { target: String },

    /// Defend location
    Defend { location: String },

    /// Move to destination
    Move { destination: String },

    /// Gather resource
    Gather { resource: String },

    /// Custom order (game-specific)
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

/// Order priority levels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Result of order execution
#[derive(Clone, Debug, PartialEq)]
pub enum OrderOutcome {
    /// Order was executed
    Executed,

    /// Order was refused
    Refused { reason: String },
}

/// Errors that can occur when issuing orders
#[derive(Debug, Clone, PartialEq)]
pub enum OrderError {
    FactionNotFound,
    MemberNotFound,
    NotDirectSubordinate,
}

impl std::fmt::Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::FactionNotFound => write!(f, "Faction not found"),
            OrderError::MemberNotFound => write!(f, "Member not found"),
            OrderError::NotDirectSubordinate => {
                write!(f, "Target is not a direct subordinate")
            }
        }
    }
}

impl std::error::Error for OrderError {}

/// Errors that can occur during promotion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromotionError {
    FactionNotFound,
    MemberNotFound,
    RankNotFound,
    NotEligible,
    CustomConditionFailed,
}

impl std::fmt::Display for PromotionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromotionError::FactionNotFound => write!(f, "Faction not found"),
            PromotionError::MemberNotFound => write!(f, "Member not found"),
            PromotionError::RankNotFound => write!(f, "Rank definition not found"),
            PromotionError::NotEligible => write!(f, "Member not eligible for promotion"),
            PromotionError::CustomConditionFailed => {
                write!(f, "Custom promotion condition failed")
            }
        }
    }
}

impl std::error::Error for PromotionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_creation() {
        let member = Member::new("m1", "John Doe", "private");

        assert_eq!(member.id, "m1");
        assert_eq!(member.name, "John Doe");
        assert_eq!(member.rank, "private");
        assert_eq!(member.superior, None);
        assert_eq!(member.loyalty, 1.0);
        assert_eq!(member.morale, 1.0);
        assert_eq!(member.tenure, 0);
    }

    #[test]
    fn test_member_builder_pattern() {
        let member = Member::new("m1", "Jane Smith", "sergeant")
            .with_superior("captain1")
            .with_loyalty(0.8)
            .with_morale(0.9)
            .with_tenure(10);

        assert_eq!(member.superior, Some("captain1".to_string()));
        assert_eq!(member.loyalty, 0.8);
        assert_eq!(member.morale, 0.9);
        assert_eq!(member.tenure, 10);
    }

    #[test]
    fn test_member_loyalty_clamping() {
        let member = Member::new("m1", "Test", "private")
            .with_loyalty(1.5) // Should clamp to 1.0
            .with_morale(-0.5); // Should clamp to 0.0

        assert_eq!(member.loyalty, 1.0);
        assert_eq!(member.morale, 0.0);
    }

    #[test]
    fn test_order_creation() {
        let order = Order::attack("enemy_base");

        match order.order_type {
            OrderType::Attack { ref target } => assert_eq!(target, "enemy_base"),
            _ => panic!("Expected Attack order"),
        }
        assert_eq!(order.priority, Priority::Normal);
    }

    #[test]
    fn test_order_with_priority() {
        let order = Order::defend("fortress").with_priority(Priority::Critical);

        assert_eq!(order.priority, Priority::Critical);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn test_order_outcome_executed() {
        let outcome = OrderOutcome::Executed;
        assert_eq!(outcome, OrderOutcome::Executed);
    }

    #[test]
    fn test_order_outcome_refused() {
        let outcome = OrderOutcome::Refused {
            reason: "Low loyalty".to_string(),
        };

        match outcome {
            OrderOutcome::Refused { ref reason } => assert_eq!(reason, "Low loyalty"),
            _ => panic!("Expected Refused outcome"),
        }
    }

    #[test]
    fn test_order_error_display() {
        let error = OrderError::FactionNotFound;
        assert_eq!(error.to_string(), "Faction not found");

        let error = OrderError::NotDirectSubordinate;
        assert_eq!(error.to_string(), "Target is not a direct subordinate");
    }

    #[test]
    fn test_promotion_error_display() {
        let error = PromotionError::NotEligible;
        assert_eq!(error.to_string(), "Member not eligible for promotion");

        let error = PromotionError::CustomConditionFailed;
        assert_eq!(error.to_string(), "Custom promotion condition failed");
    }

    #[test]
    fn test_member_serialization() {
        let member = Member::new("m1", "Test", "private").with_loyalty(0.7);

        let json = serde_json::to_string(&member).unwrap();
        let deserialized: Member = serde_json::from_str(&json).unwrap();

        assert_eq!(member, deserialized);
    }

    #[test]
    fn test_order_serialization() {
        let order = Order::attack("target1").with_priority(Priority::High);

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();

        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_custom_order() {
        let custom_data = serde_json::json!({
            "action": "patrol",
            "route": ["a", "b", "c"]
        });

        let order = Order::new(
            OrderType::Custom {
                key: "patrol_route".to_string(),
                data: custom_data.clone(),
            },
            Priority::Low,
        );

        match order.order_type {
            OrderType::Custom { ref key, ref data } => {
                assert_eq!(key, "patrol_route");
                assert_eq!(data, &custom_data);
            }
            _ => panic!("Expected Custom order"),
        }
    }
}
