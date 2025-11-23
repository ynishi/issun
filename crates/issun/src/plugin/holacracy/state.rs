//! State management for holacracy plugin
//!
//! Manages task pools, member assignments, circles, and organizational state.

use super::config::HolacracyConfig;
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Member in a holacracy organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolacracyMember {
    /// Unique identifier
    pub id: MemberId,
    /// Display name
    pub name: String,
    /// Current roles held by this member
    pub current_roles: Vec<RoleId>,
    /// Skills and proficiency levels
    pub skills: HashMap<SkillTag, SkillLevel>,
    /// Currently assigned tasks
    pub assigned_tasks: Vec<TaskId>,
    /// Autonomy level (0.0-1.0) - affects bidding
    pub autonomy_level: f32,
    /// Interest/motivation for different task types
    pub interests: HashMap<SkillTag, f32>,
    /// Last role switch timestamp (for cooldown)
    pub last_role_switch: Option<u64>,
}

impl HolacracyMember {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            current_roles: Vec::new(),
            skills: HashMap::new(),
            assigned_tasks: Vec::new(),
            autonomy_level: 0.5,
            interests: HashMap::new(),
            last_role_switch: None,
        }
    }

    /// Add a skill with proficiency level
    pub fn with_skill(mut self, skill: SkillTag, level: SkillLevel) -> Self {
        self.skills.insert(skill, level);
        self
    }

    /// Set autonomy level
    pub fn with_autonomy(mut self, level: f32) -> Self {
        self.autonomy_level = level.clamp(0.0, 1.0);
        self
    }

    /// Add interest for a skill tag
    pub fn with_interest(mut self, skill: SkillTag, interest: f32) -> Self {
        self.interests.insert(skill, interest.clamp(0.0, 1.0));
        self
    }

    /// Check if member has a specific skill
    pub fn has_skill(&self, skill: &SkillTag) -> bool {
        self.skills.contains_key(skill)
    }

    /// Get skill level (0.0 if not present)
    pub fn get_skill_level(&self, skill: &SkillTag) -> SkillLevel {
        self.skills.get(skill).copied().unwrap_or(0.0)
    }

    /// Check if member can take more tasks
    pub fn can_take_task(&self, config: &HolacracyConfig) -> bool {
        self.assigned_tasks.len() < config.max_tasks_per_member
    }

    /// Check if member can take more roles
    pub fn can_take_role(&self, config: &HolacracyConfig) -> bool {
        self.current_roles.len() < config.max_roles_per_member
    }

    /// Add a task assignment
    pub fn assign_task(&mut self, task_id: TaskId) {
        if !self.assigned_tasks.contains(&task_id) {
            self.assigned_tasks.push(task_id);
        }
    }

    /// Remove a task assignment
    pub fn unassign_task(&mut self, task_id: &str) -> bool {
        if let Some(pos) = self.assigned_tasks.iter().position(|id| id == task_id) {
            self.assigned_tasks.remove(pos);
            true
        } else {
            false
        }
    }

    /// Add a role
    pub fn add_role(&mut self, role_id: RoleId) {
        if !self.current_roles.contains(&role_id) {
            self.current_roles.push(role_id);
        }
    }

    /// Remove a role
    pub fn remove_role(&mut self, role_id: &str) -> bool {
        if let Some(pos) = self.current_roles.iter().position(|id| id == role_id) {
            self.current_roles.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get current workload (number of assigned tasks)
    pub fn workload(&self) -> usize {
        self.assigned_tasks.len()
    }
}

/// Task pool for managing available and assigned tasks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskPool {
    /// All tasks in the pool
    tasks: HashMap<TaskId, Task>,
    /// Active bids for tasks
    bids: HashMap<TaskId, Vec<Bid>>,
    /// Task dependency graph (task_id -> dependencies)
    dependencies: HashMap<TaskId, Vec<TaskId>>,
    /// Bidding start timestamps
    bidding_started: HashMap<TaskId, u64>,
}

impl TaskPool {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            bids: HashMap::new(),
            dependencies: HashMap::new(),
            bidding_started: HashMap::new(),
        }
    }

    /// Add a new task to the pool
    pub fn add_task(&mut self, task: Task) -> Result<(), HolacracyError> {
        if self.tasks.contains_key(&task.id) {
            return Err(HolacracyError::Custom(format!(
                "Task {} already exists",
                task.id
            )));
        }

        // Store dependencies
        if !task.dependencies.is_empty() {
            self.dependencies
                .insert(task.id.clone(), task.dependencies.clone());
        }

        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Get a task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Get a mutable task by ID
    pub fn get_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(task_id)
    }

    /// Remove a task
    pub fn remove_task(&mut self, task_id: &str) -> Option<Task> {
        self.bids.remove(task_id);
        self.dependencies.remove(task_id);
        self.bidding_started.remove(task_id);
        self.tasks.remove(task_id)
    }

    /// Get all available tasks (ready to be bid on)
    pub fn available_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Available || t.status == TaskStatus::Bidding)
            .collect()
    }

    /// Get all assigned tasks
    pub fn assigned_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Assigned || t.status == TaskStatus::InProgress)
            .collect()
    }

    /// Add a bid for a task
    pub fn add_bid(&mut self, bid: Bid) -> Result<(), HolacracyError> {
        if !self.tasks.contains_key(&bid.task_id) {
            return Err(HolacracyError::TaskNotFound(bid.task_id.clone()));
        }

        self.bids.entry(bid.task_id.clone()).or_default().push(bid);
        Ok(())
    }

    /// Get bids for a task
    pub fn get_bids(&self, task_id: &str) -> Vec<&Bid> {
        self.bids
            .get(task_id)
            .map(|bids| bids.iter().collect())
            .unwrap_or_default()
    }

    /// Get best bid for a task (highest score)
    pub fn get_best_bid(&self, task_id: &str) -> Option<&Bid> {
        self.bids.get(task_id).and_then(|bids| {
            bids.iter()
                .max_by(|a, b| a.score.total.partial_cmp(&b.score.total).unwrap())
        })
    }

    /// Clear bids for a task
    pub fn clear_bids(&mut self, task_id: &str) {
        self.bids.remove(task_id);
    }

    /// Start bidding for a task
    pub fn start_bidding(&mut self, task_id: &str, current_turn: u64) -> Result<(), HolacracyError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| HolacracyError::TaskNotFound(task_id.to_string()))?;

        task.status = TaskStatus::Bidding;
        self.bidding_started.insert(task_id.to_string(), current_turn);
        Ok(())
    }

    /// Check if bidding period has expired
    pub fn is_bidding_expired(&self, task_id: &str, current_turn: u64, config: &HolacracyConfig) -> bool {
        if let Some(&start_turn) = self.bidding_started.get(task_id) {
            current_turn - start_turn >= config.bidding.bidding_duration
        } else {
            false
        }
    }

    /// Assign task to member
    pub fn assign_task(&mut self, task_id: &str, member_id: &str) -> Result<(), HolacracyError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| HolacracyError::TaskNotFound(task_id.to_string()))?;

        if task.assignee.is_some() {
            return Err(HolacracyError::TaskAlreadyAssigned(task_id.to_string()));
        }

        task.status = TaskStatus::Assigned;
        task.assignee = Some(member_id.to_string());
        self.clear_bids(task_id);
        self.bidding_started.remove(task_id);
        Ok(())
    }

    /// Check if task dependencies are satisfied
    pub fn dependencies_satisfied(&self, task_id: &str) -> bool {
        if let Some(deps) = self.dependencies.get(task_id) {
            deps.iter().all(|dep_id| {
                self.tasks
                    .get(dep_id)
                    .map(|t| t.is_completed())
                    .unwrap_or(false)
            })
        } else {
            true // No dependencies
        }
    }

    /// Get number of tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Global holacracy state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolacracyState {
    /// All members in the organization
    members: HashMap<MemberId, HolacracyMember>,
    /// Task pool
    task_pool: TaskPool,
    /// All circles in the organization
    circles: HashMap<CircleId, Circle>,
    /// Current turn/tick number
    current_turn: u64,
}

impl HolacracyState {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
            task_pool: TaskPool::new(),
            circles: HashMap::new(),
            current_turn: 0,
        }
    }

    /// Add a member
    pub fn add_member(&mut self, member: HolacracyMember) {
        self.members.insert(member.id.clone(), member);
    }

    /// Get a member
    pub fn get_member(&self, member_id: &str) -> Option<&HolacracyMember> {
        self.members.get(member_id)
    }

    /// Get a mutable member
    pub fn get_member_mut(&mut self, member_id: &str) -> Option<&mut HolacracyMember> {
        self.members.get_mut(member_id)
    }

    /// Remove a member
    pub fn remove_member(&mut self, member_id: &str) -> Option<HolacracyMember> {
        self.members.remove(member_id)
    }

    /// Get all members
    pub fn all_members(&self) -> impl Iterator<Item = &HolacracyMember> {
        self.members.values()
    }

    /// Get task pool
    pub fn task_pool(&self) -> &TaskPool {
        &self.task_pool
    }

    /// Get mutable task pool
    pub fn task_pool_mut(&mut self) -> &mut TaskPool {
        &mut self.task_pool
    }

    /// Add a circle
    pub fn add_circle(&mut self, circle: Circle) {
        self.circles.insert(circle.id.clone(), circle);
    }

    /// Get a circle
    pub fn get_circle(&self, circle_id: &str) -> Option<&Circle> {
        self.circles.get(circle_id)
    }

    /// Get a mutable circle
    pub fn get_circle_mut(&mut self, circle_id: &str) -> Option<&mut Circle> {
        self.circles.get_mut(circle_id)
    }

    /// Remove a circle
    pub fn remove_circle(&mut self, circle_id: &str) -> Option<Circle> {
        self.circles.remove(circle_id)
    }

    /// Get all circles
    pub fn all_circles(&self) -> impl Iterator<Item = &Circle> {
        self.circles.values()
    }

    /// Get current turn
    pub fn current_turn(&self) -> u64 {
        self.current_turn
    }

    /// Advance turn
    pub fn advance_turn(&mut self) {
        self.current_turn += 1;
    }

    /// Get member count
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Get circle count
    pub fn circle_count(&self) -> usize {
        self.circles.len()
    }
}

impl Default for HolacracyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_creation() {
        let member = HolacracyMember::new("m1", "Alice");
        assert_eq!(member.id, "m1");
        assert_eq!(member.name, "Alice");
        assert_eq!(member.autonomy_level, 0.5);
    }

    #[test]
    fn test_member_with_skills() {
        let member = HolacracyMember::new("m1", "Alice")
            .with_skill("rust".into(), 0.8)
            .with_skill("python".into(), 0.6);

        assert_eq!(member.get_skill_level(&"rust".into()), 0.8);
        assert_eq!(member.get_skill_level(&"python".into()), 0.6);
        assert!(member.has_skill(&"rust".into()));
    }

    #[test]
    fn test_member_task_assignment() {
        let mut member = HolacracyMember::new("m1", "Alice");
        let config = HolacracyConfig::default();

        assert!(member.can_take_task(&config));
        member.assign_task("task1".to_string());
        assert_eq!(member.workload(), 1);

        member.assign_task("task2".to_string());
        assert_eq!(member.workload(), 2);

        assert!(member.unassign_task("task1"));
        assert_eq!(member.workload(), 1);
    }

    #[test]
    fn test_member_role_management() {
        let mut member = HolacracyMember::new("m1", "Alice");
        let config = HolacracyConfig::default();

        assert!(member.can_take_role(&config));
        member.add_role("role1".to_string());
        assert_eq!(member.current_roles.len(), 1);

        member.add_role("role2".to_string());
        assert_eq!(member.current_roles.len(), 2);

        assert!(member.remove_role("role1"));
        assert_eq!(member.current_roles.len(), 1);
    }

    #[test]
    fn test_task_pool_creation() {
        let pool = TaskPool::new();
        assert_eq!(pool.task_count(), 0);
    }

    #[test]
    fn test_task_pool_add_task() {
        let mut pool = TaskPool::new();
        let task = Task::new("t1", "Test task");

        assert!(pool.add_task(task).is_ok());
        assert_eq!(pool.task_count(), 1);
    }

    #[test]
    fn test_task_pool_get_task() {
        let mut pool = TaskPool::new();
        let task = Task::new("t1", "Test task");
        pool.add_task(task).unwrap();

        let retrieved = pool.get_task("t1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().description, "Test task");
    }

    #[test]
    fn test_task_pool_available_tasks() {
        let mut pool = TaskPool::new();
        pool.add_task(Task::new("t1", "Available")).unwrap();

        let mut task2 = Task::new("t2", "Assigned");
        task2.status = TaskStatus::Assigned;
        pool.add_task(task2).unwrap();

        let available = pool.available_tasks();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "t1");
    }

    #[test]
    fn test_task_pool_bidding() {
        let mut pool = TaskPool::new();
        pool.add_task(Task::new("t1", "Task")).unwrap();

        let bid = Bid::new("t1", "alice", BidScore::new(0.8, 0.6, 0.9), 50);
        assert!(pool.add_bid(bid).is_ok());

        let bids = pool.get_bids("t1");
        assert_eq!(bids.len(), 1);
    }

    #[test]
    fn test_task_pool_best_bid() {
        let mut pool = TaskPool::new();
        pool.add_task(Task::new("t1", "Task")).unwrap();

        let bid1 = Bid::new("t1", "alice", BidScore::new(0.8, 0.6, 0.9), 50);
        let bid2 = Bid::new("t1", "bob", BidScore::new(0.6, 0.5, 0.7), 60);
        pool.add_bid(bid1).unwrap();
        pool.add_bid(bid2).unwrap();

        let best = pool.get_best_bid("t1");
        assert!(best.is_some());
        assert_eq!(best.unwrap().member_id, "alice");
    }

    #[test]
    fn test_task_pool_assign_task() {
        let mut pool = TaskPool::new();
        pool.add_task(Task::new("t1", "Task")).unwrap();

        assert!(pool.assign_task("t1", "alice").is_ok());

        let task = pool.get_task("t1").unwrap();
        assert_eq!(task.status, TaskStatus::Assigned);
        assert_eq!(task.assignee, Some("alice".to_string()));
    }

    #[test]
    fn test_task_pool_dependencies() {
        let mut pool = TaskPool::new();

        let mut task1 = Task::new("t1", "First");
        task1.status = TaskStatus::Completed;
        pool.add_task(task1).unwrap();

        let task2 = Task::new("t2", "Second").with_dependencies(vec!["t1".to_string()]);
        pool.add_task(task2).unwrap();

        assert!(pool.dependencies_satisfied("t2"));
    }

    #[test]
    fn test_holacracy_state_creation() {
        let state = HolacracyState::new();
        assert_eq!(state.member_count(), 0);
        assert_eq!(state.circle_count(), 0);
        assert_eq!(state.current_turn(), 0);
    }

    #[test]
    fn test_holacracy_state_add_member() {
        let mut state = HolacracyState::new();
        let member = HolacracyMember::new("m1", "Alice");

        state.add_member(member);
        assert_eq!(state.member_count(), 1);
    }

    #[test]
    fn test_holacracy_state_add_circle() {
        let mut state = HolacracyState::new();
        let circle = Circle::new("c1", "Engineering", "Build products");

        state.add_circle(circle);
        assert_eq!(state.circle_count(), 1);
    }

    #[test]
    fn test_holacracy_state_advance_turn() {
        let mut state = HolacracyState::new();
        assert_eq!(state.current_turn(), 0);

        state.advance_turn();
        assert_eq!(state.current_turn(), 1);

        state.advance_turn();
        assert_eq!(state.current_turn(), 2);
    }

    #[test]
    fn test_member_autonomy_clamping() {
        let member = HolacracyMember::new("m1", "Alice").with_autonomy(1.5);
        assert_eq!(member.autonomy_level, 1.0);

        let member2 = HolacracyMember::new("m2", "Bob").with_autonomy(-0.5);
        assert_eq!(member2.autonomy_level, 0.0);
    }
}
