//! Service functions for task assignment and bid scoring
//!
//! Implements pure algorithms for calculating bid scores, matching tasks to members,
//! and optimizing task allocation.

use super::config::HolacracyConfig;
use super::state::{HolacracyMember, TaskPool};
use super::types::*;
use std::collections::HashSet;

/// Service for task assignment algorithms
pub struct TaskAssignmentService;

impl TaskAssignmentService {
    /// Calculate bid score for a member bidding on a task
    ///
    /// Score components:
    /// - Skill match: How well member's skills match task requirements
    /// - Workload factor: How much capacity member has (lower workload = higher score)
    /// - Interest factor: Member's interest in the task's skill tags
    pub fn calculate_bid_score(
        member: &HolacracyMember,
        task: &Task,
        config: &HolacracyConfig,
    ) -> Result<BidScore, HolacracyError> {
        // Calculate skill match
        let skill_match = Self::calculate_skill_match(member, task);

        // Calculate workload factor (inverse of current load)
        let workload_factor = if config.max_tasks_per_member > 0 {
            1.0 - (member.workload() as f32 / config.max_tasks_per_member as f32)
        } else {
            0.0
        };

        // Calculate interest factor
        let interest_factor = Self::calculate_interest_factor(member, task);

        // Weighted combination
        let total = (skill_match * config.skill_match_weight)
            + (workload_factor * config.workload_weight)
            + (interest_factor * config.interest_weight);

        Ok(BidScore {
            total,
            skill_match,
            workload_factor,
            interest_factor,
        })
    }

    /// Calculate how well member's skills match task requirements
    ///
    /// Returns 0.0-1.0 score:
    /// - 1.0 = perfect match (all skills at high level)
    /// - 0.0 = no matching skills
    fn calculate_skill_match(member: &HolacracyMember, task: &Task) -> f32 {
        if task.required_skills.is_empty() {
            return 1.0; // Task requires no skills
        }

        let mut total_match = 0.0;
        let mut skill_count = 0;

        for required_skill in &task.required_skills {
            let skill_level = member.get_skill_level(required_skill);
            total_match += skill_level;
            skill_count += 1;
        }

        if skill_count > 0 {
            total_match / skill_count as f32
        } else {
            0.0
        }
    }

    /// Calculate member's interest in the task
    ///
    /// Returns 0.0-1.0 score based on member's declared interests
    fn calculate_interest_factor(member: &HolacracyMember, task: &Task) -> f32 {
        if task.required_skills.is_empty() {
            return 0.5; // Neutral interest for skill-less tasks
        }

        let mut total_interest = 0.0;
        let mut skill_count = 0;

        for required_skill in &task.required_skills {
            if let Some(&interest) = member.interests.get(required_skill) {
                total_interest += interest;
                skill_count += 1;
            }
        }

        if skill_count > 0 {
            total_interest / skill_count as f32
        } else {
            0.0 // No declared interest
        }
    }

    /// Check if member meets minimum requirements to bid on task
    pub fn can_bid_on_task(
        member: &HolacracyMember,
        task: &Task,
        config: &HolacracyConfig,
    ) -> Result<(), String> {
        // Check workload capacity
        if !member.can_take_task(config) {
            return Err(format!(
                "Member {} has reached max task capacity ({}/{})",
                member.id,
                member.workload(),
                config.max_tasks_per_member
            ));
        }

        // Check minimum skill level
        for required_skill in &task.required_skills {
            let skill_level = member.get_skill_level(required_skill);
            if skill_level < config.min_skill_level_for_bid {
                return Err(format!(
                    "Member {} skill level {} for '{}' is below minimum {}",
                    member.id,
                    skill_level,
                    required_skill.as_str(),
                    config.min_skill_level_for_bid
                ));
            }
        }

        Ok(())
    }

    /// Find best matches for a task from available members
    ///
    /// Returns up to `limit` members sorted by bid score (highest first)
    pub fn find_best_matches(
        task: &Task,
        members: &[&HolacracyMember],
        config: &HolacracyConfig,
        limit: usize,
    ) -> Vec<(MemberId, BidScore)> {
        let mut matches: Vec<(MemberId, BidScore)> = members
            .iter()
            .filter(|m| Self::can_bid_on_task(m, task, config).is_ok())
            .filter_map(|m| {
                Self::calculate_bid_score(m, task, config)
                    .ok()
                    .map(|score| (m.id.clone(), score))
            })
            .collect();

        // Sort by total score descending
        matches.sort_by(|a, b| b.1.total.partial_cmp(&a.1.total).unwrap());

        // Take top N
        matches.into_iter().take(limit).collect()
    }

    /// Estimate task completion time based on member skills
    ///
    /// Better skill match = faster completion
    pub fn estimate_completion_time(member: &HolacracyMember, task: &Task, base_cost: f32) -> u64 {
        let skill_match = Self::calculate_skill_match(member, task);

        // Adjust cost based on skill match
        // High skill (1.0) = 100% efficiency (base cost)
        // Low skill (0.0) = 200% time needed
        let efficiency = 0.5 + (skill_match * 0.5); // 0.5 to 1.0
        let adjusted_cost = base_cost / efficiency;

        adjusted_cost.ceil() as u64
    }

    /// Check for circular dependencies in task graph
    pub fn has_circular_dependency(
        task_pool: &TaskPool,
        task_id: &str,
        visited: &mut HashSet<TaskId>,
    ) -> bool {
        if visited.contains(task_id) {
            return true; // Cycle detected
        }

        visited.insert(task_id.to_string());

        if let Some(task) = task_pool.get_task(task_id) {
            for dep_id in &task.dependencies {
                if Self::has_circular_dependency(task_pool, dep_id, visited) {
                    return true;
                }
            }
        }

        visited.remove(task_id);
        false
    }

    /// Get all tasks that are ready to be worked on (no blocking dependencies)
    pub fn get_ready_tasks(task_pool: &TaskPool) -> Vec<TaskId> {
        task_pool
            .available_tasks()
            .into_iter()
            .filter(|t| task_pool.dependencies_satisfied(&t.id))
            .map(|t| t.id.clone())
            .collect()
    }

    /// Calculate priority score for task (higher = more urgent)
    pub fn calculate_priority_score(task: &Task, config: &HolacracyConfig) -> f32 {
        let mut score = task.priority.value() as f32;

        // Boost critical tasks
        if task.priority == TaskPriority::Critical {
            score *= config.critical_priority_boost;
        }

        // Factor in deadline urgency (if any)
        // This is a simplified version - in real system would use current_turn
        if task.deadline.is_some() {
            score *= 1.2;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_member() -> HolacracyMember {
        HolacracyMember::new("m1", "Alice")
            .with_skill("rust".into(), 0.8)
            .with_skill("python".into(), 0.6)
            .with_autonomy(0.7)
            .with_interest("rust".into(), 0.9)
    }

    fn create_test_task() -> Task {
        Task::new("t1", "Implement feature")
            .with_skills(vec!["rust".into()])
            .with_cost(5.0)
            .with_priority(TaskPriority::High)
    }

    #[test]
    fn test_calculate_skill_match() {
        let member = create_test_member();
        let task = create_test_task();

        let skill_match = TaskAssignmentService::calculate_skill_match(&member, &task);
        assert_eq!(skill_match, 0.8); // Member has 0.8 in rust
    }

    #[test]
    fn test_calculate_skill_match_multiple_skills() {
        let member = create_test_member();
        let task = Task::new("t1", "Task").with_skills(vec!["rust".into(), "python".into()]);

        let skill_match = TaskAssignmentService::calculate_skill_match(&member, &task);
        assert!((skill_match - 0.7).abs() < 0.01); // (0.8 + 0.6) / 2 = 0.7
    }

    #[test]
    fn test_calculate_skill_match_no_skills_required() {
        let member = create_test_member();
        let task = Task::new("t1", "Simple task"); // No skills required

        let skill_match = TaskAssignmentService::calculate_skill_match(&member, &task);
        assert_eq!(skill_match, 1.0); // Perfect match for skill-less tasks
    }

    #[test]
    fn test_calculate_interest_factor() {
        let member = create_test_member();
        let task = create_test_task();

        let interest = TaskAssignmentService::calculate_interest_factor(&member, &task);
        assert_eq!(interest, 0.9); // Member has 0.9 interest in rust
    }

    #[test]
    fn test_calculate_bid_score() {
        let member = create_test_member();
        let task = create_test_task();
        let config = HolacracyConfig::default();

        let score = TaskAssignmentService::calculate_bid_score(&member, &task, &config);
        assert!(score.is_ok());

        let bid_score = score.unwrap();
        assert_eq!(bid_score.skill_match, 0.8);
        assert!(bid_score.workload_factor > 0.9); // No tasks assigned yet
        assert_eq!(bid_score.interest_factor, 0.9);
    }

    #[test]
    fn test_can_bid_on_task_success() {
        let member = create_test_member();
        let task = create_test_task();
        let config = HolacracyConfig::default();

        let result = TaskAssignmentService::can_bid_on_task(&member, &task, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_bid_on_task_workload_exceeded() {
        let mut member = create_test_member();
        let task = create_test_task();
        let config = HolacracyConfig::default().with_max_tasks(2);

        // Assign max tasks
        member.assign_task("t1".to_string());
        member.assign_task("t2".to_string());

        let result = TaskAssignmentService::can_bid_on_task(&member, &task, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_bid_on_task_insufficient_skill() {
        let member = HolacracyMember::new("m1", "Novice").with_skill("rust".into(), 0.2); // Below default min (0.3)

        let task = create_test_task();
        let config = HolacracyConfig::default();

        let result = TaskAssignmentService::can_bid_on_task(&member, &task, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_best_matches() {
        let member1 = HolacracyMember::new("m1", "Expert")
            .with_skill("rust".into(), 0.9)
            .with_autonomy(0.8);

        let member2 = HolacracyMember::new("m2", "Intermediate")
            .with_skill("rust".into(), 0.6)
            .with_autonomy(0.7);

        let member3 = HolacracyMember::new("m3", "Novice")
            .with_skill("rust".into(), 0.3)
            .with_autonomy(0.5);

        let members = vec![&member1, &member2, &member3];
        let task = create_test_task();
        let config = HolacracyConfig::default();

        let matches = TaskAssignmentService::find_best_matches(&task, &members, &config, 3);

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].0, "m1"); // Best match
        assert_eq!(matches[1].0, "m2");
        assert_eq!(matches[2].0, "m3");
    }

    #[test]
    fn test_estimate_completion_time() {
        let expert = HolacracyMember::new("expert", "Expert").with_skill("rust".into(), 1.0);

        let novice = HolacracyMember::new("novice", "Novice").with_skill("rust".into(), 0.0);

        let task = create_test_task().with_cost(10.0);

        let expert_time = TaskAssignmentService::estimate_completion_time(&expert, &task, 10.0);
        let novice_time = TaskAssignmentService::estimate_completion_time(&novice, &task, 10.0);

        assert_eq!(expert_time, 10); // 100% efficiency
        assert_eq!(novice_time, 20); // 50% efficiency (takes 2x time)
    }

    #[test]
    fn test_has_circular_dependency() {
        let mut pool = TaskPool::new();

        let task1 = Task::new("t1", "First").with_dependencies(vec!["t2".to_string()]);
        let task2 = Task::new("t2", "Second").with_dependencies(vec!["t1".to_string()]);

        pool.add_task(task1).unwrap();
        pool.add_task(task2).unwrap();

        let mut visited = HashSet::new();
        assert!(TaskAssignmentService::has_circular_dependency(
            &pool,
            "t1",
            &mut visited
        ));
    }

    #[test]
    fn test_no_circular_dependency() {
        let mut pool = TaskPool::new();

        let task1 = Task::new("t1", "First");
        let task2 = Task::new("t2", "Second").with_dependencies(vec!["t1".to_string()]);

        pool.add_task(task1).unwrap();
        pool.add_task(task2).unwrap();

        let mut visited = HashSet::new();
        assert!(!TaskAssignmentService::has_circular_dependency(
            &pool,
            "t2",
            &mut visited
        ));
    }

    #[test]
    fn test_calculate_priority_score() {
        let config = HolacracyConfig::default();

        let critical_task = Task::new("t1", "Critical").with_priority(TaskPriority::Critical);
        let high_task = Task::new("t2", "High").with_priority(TaskPriority::High);
        let medium_task = Task::new("t3", "Medium").with_priority(TaskPriority::Medium);

        let critical_score =
            TaskAssignmentService::calculate_priority_score(&critical_task, &config);
        let high_score = TaskAssignmentService::calculate_priority_score(&high_task, &config);
        let medium_score = TaskAssignmentService::calculate_priority_score(&medium_task, &config);

        assert!(critical_score > high_score);
        assert!(high_score > medium_score);
    }

    #[test]
    fn test_get_ready_tasks() {
        let mut pool = TaskPool::new();

        let mut task1 = Task::new("t1", "First");
        task1.status = TaskStatus::Completed;
        pool.add_task(task1).unwrap();

        let task2 = Task::new("t2", "Second").with_dependencies(vec!["t1".to_string()]);
        pool.add_task(task2).unwrap();

        let task3 = Task::new("t3", "Third"); // No dependencies
        pool.add_task(task3).unwrap();

        let ready = TaskAssignmentService::get_ready_tasks(&pool);
        assert_eq!(ready.len(), 2); // t2 and t3 are ready
        assert!(ready.contains(&"t2".to_string()));
        assert!(ready.contains(&"t3".to_string()));
    }
}
