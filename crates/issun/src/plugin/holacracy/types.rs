//! Core type definitions for holacracy plugin
//!
//! Defines tasks, roles, bids, circles, and related types for task-based
//! self-organizing systems.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for a task
pub type TaskId = String;

/// Unique identifier for a role
pub type RoleId = String;

/// Unique identifier for a circle
pub type CircleId = String;

/// Unique identifier for a member
pub type MemberId = String;

/// Skill tag for matching tasks and members
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillTag(pub String);

impl SkillTag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SkillTag {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SkillTag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Skill proficiency level (0.0-1.0)
pub type SkillLevel = f32;

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TaskPriority {
    /// Urgent and important
    Critical,
    /// Important but not urgent
    High,
    /// Normal priority
    #[default]
    Medium,
    /// Can be delayed
    Low,
}

impl TaskPriority {
    /// Get numeric value for comparison (higher = more urgent)
    pub fn value(&self) -> u8 {
        match self {
            TaskPriority::Critical => 4,
            TaskPriority::High => 3,
            TaskPriority::Medium => 2,
            TaskPriority::Low => 1,
        }
    }
}

/// Status of a task in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    /// Available in task pool
    #[default]
    Available,
    /// Being bid on by members
    Bidding,
    /// Assigned to a member
    Assigned,
    /// Work in progress
    InProgress,
    /// Blocked by dependencies
    Blocked,
    /// Successfully completed
    Completed,
    /// Cancelled or abandoned
    Cancelled,
}

/// Type of role in the organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleType {
    /// Operational role (execution)
    Operational {
        /// Skills required for this role
        required_skills: HashSet<SkillTag>,
    },
    /// Coordination role (facilitation)
    Coordination {
        /// Circle this role coordinates
        circle_id: CircleId,
    },
    /// Representative link between circles
    RepLink {
        /// Source circle
        from_circle: CircleId,
        /// Target circle
        to_circle: CircleId,
    },
    /// Custom role type
    Custom(String),
}

impl RoleType {
    pub fn operational(skills: Vec<SkillTag>) -> Self {
        RoleType::Operational {
            required_skills: skills.into_iter().collect(),
        }
    }

    pub fn coordination(circle_id: impl Into<String>) -> Self {
        RoleType::Coordination {
            circle_id: circle_id.into(),
        }
    }

    pub fn rep_link(from: impl Into<String>, to: impl Into<String>) -> Self {
        RoleType::RepLink {
            from_circle: from.into(),
            to_circle: to.into(),
        }
    }
}

/// A task in the task pool
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: TaskId,
    /// Human-readable description
    pub description: String,
    /// Priority level
    pub priority: TaskPriority,
    /// Current status
    pub status: TaskStatus,
    /// Required skills
    pub required_skills: HashSet<SkillTag>,
    /// Estimated cost/effort
    pub estimated_cost: f32,
    /// Task dependencies (must be completed first)
    pub dependencies: Vec<TaskId>,
    /// Currently assigned member (if any)
    pub assignee: Option<MemberId>,
    /// When task was created
    pub created_at: u64,
    /// Deadline (if any)
    pub deadline: Option<u64>,
}

impl Task {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            priority: TaskPriority::default(),
            status: TaskStatus::default(),
            required_skills: HashSet::new(),
            estimated_cost: 1.0,
            dependencies: Vec::new(),
            assignee: None,
            created_at: 0,
            deadline: None,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_skills(mut self, skills: Vec<SkillTag>) -> Self {
        self.required_skills = skills.into_iter().collect();
        self
    }

    pub fn with_cost(mut self, cost: f32) -> Self {
        self.estimated_cost = cost;
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<TaskId>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Check if task is ready to be worked on (no blocking dependencies)
    pub fn is_ready(&self) -> bool {
        self.status == TaskStatus::Available || self.status == TaskStatus::Bidding
    }

    /// Check if task is blocked
    pub fn is_blocked(&self) -> bool {
        self.status == TaskStatus::Blocked
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }
}

/// A role that can be filled by members
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier
    pub id: RoleId,
    /// Human-readable name
    pub name: String,
    /// Type of role
    pub role_type: RoleType,
    /// Accountabilities/purposes
    pub accountabilities: Vec<String>,
    /// Current holder (if filled)
    pub current_holder: Option<MemberId>,
}

impl Role {
    pub fn new(id: impl Into<String>, name: impl Into<String>, role_type: RoleType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role_type,
            accountabilities: Vec::new(),
            current_holder: None,
        }
    }

    pub fn with_accountabilities(mut self, accountabilities: Vec<String>) -> Self {
        self.accountabilities = accountabilities;
        self
    }

    pub fn with_holder(mut self, holder: impl Into<String>) -> Self {
        self.current_holder = Some(holder.into());
        self
    }

    pub fn is_filled(&self) -> bool {
        self.current_holder.is_some()
    }
}

/// Bid score calculation result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BidScore {
    /// Overall bid score (0.0-1.0)
    pub total: f32,
    /// Skill match component
    pub skill_match: f32,
    /// Workload component
    pub workload_factor: f32,
    /// Interest/motivation component
    pub interest_factor: f32,
}

impl BidScore {
    pub fn new(skill_match: f32, workload_factor: f32, interest_factor: f32) -> Self {
        let total = (skill_match * 0.5) + (workload_factor * 0.3) + (interest_factor * 0.2);
        Self {
            total,
            skill_match,
            workload_factor,
            interest_factor,
        }
    }
}

/// A bid for a task by a member
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    /// Task being bid on
    pub task_id: TaskId,
    /// Member making the bid
    pub member_id: MemberId,
    /// Bid score
    pub score: BidScore,
    /// Estimated completion time
    pub estimated_completion: u64,
    /// Bid timestamp
    pub bid_at: u64,
}

impl Bid {
    pub fn new(
        task_id: impl Into<String>,
        member_id: impl Into<String>,
        score: BidScore,
        estimated_completion: u64,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            member_id: member_id.into(),
            score,
            estimated_completion,
            bid_at: 0,
        }
    }

    /// Check if this bid is better than another
    pub fn is_better_than(&self, other: &Bid) -> bool {
        self.score.total > other.score.total
    }
}

/// A circle (team/domain) in the organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// Unique identifier
    pub id: CircleId,
    /// Human-readable name
    pub name: String,
    /// Purpose/mission
    pub purpose: String,
    /// Parent circle (if any)
    pub parent_circle: Option<CircleId>,
    /// Member IDs in this circle
    pub members: HashSet<MemberId>,
    /// Roles defined in this circle
    pub roles: HashMap<RoleId, Role>,
    /// Domains/authorities
    pub domains: Vec<String>,
}

impl Circle {
    pub fn new(id: impl Into<String>, name: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            purpose: purpose.into(),
            parent_circle: None,
            members: HashSet::new(),
            roles: HashMap::new(),
            domains: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent_circle = Some(parent.into());
        self
    }

    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.domains = domains;
        self
    }

    pub fn add_member(&mut self, member_id: impl Into<String>) -> bool {
        self.members.insert(member_id.into())
    }

    pub fn remove_member(&mut self, member_id: &str) -> bool {
        self.members.remove(member_id)
    }

    pub fn add_role(&mut self, role: Role) -> Option<Role> {
        self.roles.insert(role.id.clone(), role)
    }

    pub fn remove_role(&mut self, role_id: &str) -> Option<Role> {
        self.roles.remove(role_id)
    }

    pub fn get_role(&self, role_id: &str) -> Option<&Role> {
        self.roles.get(role_id)
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn role_count(&self) -> usize {
        self.roles.len()
    }
}

/// Errors that can occur in holacracy operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HolacracyError {
    /// Task not found
    TaskNotFound(TaskId),
    /// Role not found
    RoleNotFound(RoleId),
    /// Circle not found
    CircleNotFound(CircleId),
    /// Member not found
    MemberNotFound(MemberId),
    /// Task already assigned
    TaskAlreadyAssigned(TaskId),
    /// Role already filled
    RoleAlreadyFilled(RoleId),
    /// Circular dependency detected
    CircularDependency(TaskId),
    /// Invalid skill level (must be 0.0-1.0)
    InvalidSkillLevel(f32),
    /// Invalid cost (must be positive)
    InvalidCost(f32),
    /// Custom error message
    Custom(String),
}

impl fmt::Display for HolacracyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HolacracyError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            HolacracyError::RoleNotFound(id) => write!(f, "Role not found: {}", id),
            HolacracyError::CircleNotFound(id) => write!(f, "Circle not found: {}", id),
            HolacracyError::MemberNotFound(id) => write!(f, "Member not found: {}", id),
            HolacracyError::TaskAlreadyAssigned(id) => write!(f, "Task already assigned: {}", id),
            HolacracyError::RoleAlreadyFilled(id) => write!(f, "Role already filled: {}", id),
            HolacracyError::CircularDependency(id) => {
                write!(f, "Circular dependency detected: {}", id)
            }
            HolacracyError::InvalidSkillLevel(level) => {
                write!(f, "Invalid skill level: {} (must be 0.0-1.0)", level)
            }
            HolacracyError::InvalidCost(cost) => {
                write!(f, "Invalid cost: {} (must be positive)", cost)
            }
            HolacracyError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for HolacracyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tag_creation() {
        let tag1 = SkillTag::new("rust");
        let tag2: SkillTag = "rust".into();
        let tag3: SkillTag = "rust".to_string().into();

        assert_eq!(tag1, tag2);
        assert_eq!(tag2, tag3);
        assert_eq!(tag1.as_str(), "rust");
    }

    #[test]
    fn test_task_priority_value() {
        assert_eq!(TaskPriority::Critical.value(), 4);
        assert_eq!(TaskPriority::High.value(), 3);
        assert_eq!(TaskPriority::Medium.value(), 2);
        assert_eq!(TaskPriority::Low.value(), 1);
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical.value() > TaskPriority::High.value());
        assert!(TaskPriority::High.value() > TaskPriority::Medium.value());
        assert!(TaskPriority::Medium.value() > TaskPriority::Low.value());
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("task1", "Implement feature X");
        assert_eq!(task.id, "task1");
        assert_eq!(task.description, "Implement feature X");
        assert_eq!(task.priority, TaskPriority::Medium);
        assert_eq!(task.status, TaskStatus::Available);
        assert!(task.required_skills.is_empty());
    }

    #[test]
    fn test_task_builder() {
        let task = Task::new("task1", "Fix bug")
            .with_priority(TaskPriority::Critical)
            .with_skills(vec!["rust".into(), "debugging".into()])
            .with_cost(3.0)
            .with_dependencies(vec!["task0".to_string()])
            .with_deadline(100);

        assert_eq!(task.priority, TaskPriority::Critical);
        assert_eq!(task.required_skills.len(), 2);
        assert_eq!(task.estimated_cost, 3.0);
        assert_eq!(task.dependencies.len(), 1);
        assert_eq!(task.deadline, Some(100));
    }

    #[test]
    fn test_task_status_checks() {
        let mut task = Task::new("t1", "test");

        task.status = TaskStatus::Available;
        assert!(task.is_ready());
        assert!(!task.is_blocked());
        assert!(!task.is_completed());

        task.status = TaskStatus::Blocked;
        assert!(!task.is_ready());
        assert!(task.is_blocked());
        assert!(!task.is_completed());

        task.status = TaskStatus::Completed;
        assert!(!task.is_ready());
        assert!(!task.is_blocked());
        assert!(task.is_completed());
    }

    #[test]
    fn test_role_type_creation() {
        let op_role = RoleType::operational(vec!["coding".into(), "testing".into()]);
        let coord_role = RoleType::coordination("circle1");
        let link_role = RoleType::rep_link("circle1", "circle2");

        match op_role {
            RoleType::Operational { required_skills } => assert_eq!(required_skills.len(), 2),
            _ => panic!("Wrong role type"),
        }

        match coord_role {
            RoleType::Coordination { circle_id } => assert_eq!(circle_id, "circle1"),
            _ => panic!("Wrong role type"),
        }

        match link_role {
            RoleType::RepLink {
                from_circle,
                to_circle,
            } => {
                assert_eq!(from_circle, "circle1");
                assert_eq!(to_circle, "circle2");
            }
            _ => panic!("Wrong role type"),
        }
    }

    #[test]
    fn test_role_creation() {
        let role = Role::new(
            "role1",
            "Developer",
            RoleType::operational(vec!["coding".into()]),
        );
        assert_eq!(role.id, "role1");
        assert_eq!(role.name, "Developer");
        assert!(!role.is_filled());
    }

    #[test]
    fn test_role_builder() {
        let role = Role::new(
            "role1",
            "Developer",
            RoleType::operational(vec!["coding".into()]),
        )
        .with_accountabilities(vec![
            "Write clean code".to_string(),
            "Review PRs".to_string(),
        ])
        .with_holder("alice");

        assert_eq!(role.accountabilities.len(), 2);
        assert!(role.is_filled());
        assert_eq!(role.current_holder, Some("alice".to_string()));
    }

    #[test]
    fn test_bid_score_calculation() {
        let score = BidScore::new(0.8, 0.6, 0.9);
        // 0.8*0.5 + 0.6*0.3 + 0.9*0.2 = 0.4 + 0.18 + 0.18 = 0.76
        assert!((score.total - 0.76).abs() < 0.01);
    }

    #[test]
    fn test_bid_creation() {
        let score = BidScore::new(0.8, 0.6, 0.9);
        let bid = Bid::new("task1", "alice", score, 50);

        assert_eq!(bid.task_id, "task1");
        assert_eq!(bid.member_id, "alice");
        assert_eq!(bid.estimated_completion, 50);
    }

    #[test]
    fn test_bid_comparison() {
        let score1 = BidScore::new(0.8, 0.6, 0.9); // 0.76
        let score2 = BidScore::new(0.6, 0.5, 0.7); // 0.59
        let bid1 = Bid::new("task1", "alice", score1, 50);
        let bid2 = Bid::new("task1", "bob", score2, 60);

        assert!(bid1.is_better_than(&bid2));
        assert!(!bid2.is_better_than(&bid1));
    }

    #[test]
    fn test_circle_creation() {
        let circle = Circle::new("c1", "Engineering", "Build great products");
        assert_eq!(circle.id, "c1");
        assert_eq!(circle.name, "Engineering");
        assert_eq!(circle.purpose, "Build great products");
        assert!(circle.parent_circle.is_none());
    }

    #[test]
    fn test_circle_builder() {
        let circle = Circle::new("c1", "Backend", "Build APIs")
            .with_parent("engineering")
            .with_domains(vec!["Database".to_string(), "API".to_string()]);

        assert_eq!(circle.parent_circle, Some("engineering".to_string()));
        assert_eq!(circle.domains.len(), 2);
    }

    #[test]
    fn test_circle_member_management() {
        let mut circle = Circle::new("c1", "Team", "Purpose");

        assert_eq!(circle.member_count(), 0);

        circle.add_member("alice");
        circle.add_member("bob");
        assert_eq!(circle.member_count(), 2);

        circle.remove_member("alice");
        assert_eq!(circle.member_count(), 1);
        assert!(circle.members.contains("bob"));
    }

    #[test]
    fn test_circle_role_management() {
        let mut circle = Circle::new("c1", "Team", "Purpose");

        let role1 = Role::new("r1", "Developer", RoleType::operational(vec![]));
        let role2 = Role::new("r2", "Tester", RoleType::operational(vec![]));

        circle.add_role(role1);
        circle.add_role(role2);
        assert_eq!(circle.role_count(), 2);

        let retrieved = circle.get_role("r1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Developer");

        circle.remove_role("r1");
        assert_eq!(circle.role_count(), 1);
        assert!(circle.get_role("r1").is_none());
    }

    #[test]
    fn test_holacracy_error_display() {
        let err1 = HolacracyError::TaskNotFound("t1".to_string());
        assert_eq!(err1.to_string(), "Task not found: t1");

        let err2 = HolacracyError::InvalidSkillLevel(1.5);
        assert_eq!(
            err2.to_string(),
            "Invalid skill level: 1.5 (must be 0.0-1.0)"
        );

        let err3 = HolacracyError::Custom("Custom error".to_string());
        assert_eq!(err3.to_string(), "Custom error");
    }

    #[test]
    fn test_task_status_default() {
        assert_eq!(TaskStatus::default(), TaskStatus::Available);
    }

    #[test]
    fn test_task_priority_default() {
        assert_eq!(TaskPriority::default(), TaskPriority::Medium);
    }
}
