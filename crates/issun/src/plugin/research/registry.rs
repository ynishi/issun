//! Research registry and queue management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::*;

/// Progress model for research projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressModel {
    /// Fixed progress per turn (e.g., 0.1 per turn = 10 turns)
    TurnBased,

    /// Real-time progress (requires GameTimer plugin)
    TimeBased,

    /// Manual progress updates via events
    Manual,
}

impl Default for ProgressModel {
    fn default() -> Self {
        Self::TurnBased
    }
}

/// Configuration for research system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    /// Allow multiple projects to be researched simultaneously
    pub allow_parallel_research: bool,

    /// Maximum number of parallel research slots
    pub max_parallel_slots: usize,

    /// Progress model (turn-based, time-based, manual)
    pub progress_model: ProgressModel,

    /// Auto-advance progress each turn/tick
    pub auto_advance: bool,

    /// Base progress per turn (when auto_advance = true)
    pub base_progress_per_turn: f32,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            allow_parallel_research: false,
            max_parallel_slots: 1,
            progress_model: ProgressModel::TurnBased,
            auto_advance: true,
            base_progress_per_turn: 0.1, // 10 turns by default
        }
    }
}

/// Registry of all research projects in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRegistry {
    /// All defined research projects
    projects: HashMap<ResearchId, ResearchProject>,

    /// Current research queue (ordered list of project IDs)
    queue: Vec<ResearchId>,

    /// Configuration
    config: ResearchConfig,
}

impl Default for ResearchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResearchRegistry {
    /// Create a new research registry
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            queue: Vec::new(),
            config: ResearchConfig::default(),
        }
    }

    /// Create a registry with custom config
    pub fn with_config(config: ResearchConfig) -> Self {
        Self {
            projects: HashMap::new(),
            queue: Vec::new(),
            config,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &ResearchConfig {
        &self.config
    }

    /// Set the configuration
    pub fn set_config(&mut self, config: ResearchConfig) {
        self.config = config;
    }

    /// Add a research project definition to the registry
    pub fn define(&mut self, project: ResearchProject) {
        self.projects.insert(project.id.clone(), project);
    }

    /// Get a project by id
    pub fn get(&self, id: &ResearchId) -> Option<&ResearchProject> {
        self.projects.get(id)
    }

    /// Get a mutable reference to a project
    pub fn get_mut(&mut self, id: &ResearchId) -> Option<&mut ResearchProject> {
        self.projects.get_mut(id)
    }

    /// Queue a project for research
    ///
    /// Returns error if project doesn't exist or is already queued/completed.
    pub fn queue(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let project = self
            .projects
            .get_mut(id)
            .ok_or(ResearchError::NotFound)?;

        match project.status {
            ResearchStatus::Available => {
                project.status = ResearchStatus::Queued;
                self.queue.push(id.clone());
                // Try to activate if slots available
                self.activate_next_queued();
                Ok(())
            }
            ResearchStatus::Queued | ResearchStatus::InProgress => {
                Err(ResearchError::AlreadyQueued)
            }
            ResearchStatus::Completed => Err(ResearchError::AlreadyCompleted),
            ResearchStatus::Failed => {
                // Allow retrying failed research
                project.status = ResearchStatus::Queued;
                project.progress = 0.0;
                self.queue.push(id.clone());
                self.activate_next_queued();
                Ok(())
            }
        }
    }

    /// Get currently active research projects
    pub fn active_research(&self) -> Vec<&ResearchProject> {
        self.projects
            .values()
            .filter(|p| p.status == ResearchStatus::InProgress)
            .collect()
    }

    /// Get queued research projects (in order)
    pub fn queued_research(&self) -> Vec<&ResearchProject> {
        self.queue
            .iter()
            .filter_map(|id| self.projects.get(id))
            .filter(|p| p.status == ResearchStatus::Queued)
            .collect()
    }

    /// Advance progress for active research projects
    ///
    /// Returns IDs of completed projects.
    pub fn advance_progress(&mut self, amount: f32) -> Vec<ResearchId> {
        let mut completed = Vec::new();

        for project in self.projects.values_mut() {
            if project.status == ResearchStatus::InProgress {
                // Delegate to Service for pure calculation logic
                project.progress = super::service::ResearchService::add_progress(
                    project.progress,
                    amount,
                );

                if project.progress >= 1.0 {
                    project.status = ResearchStatus::Completed;
                    completed.push(project.id.clone());
                }
            }
        }

        // Remove completed projects from queue
        self.queue.retain(|id| !completed.contains(id));

        // Start next queued projects if slots available
        self.activate_next_queued();

        completed
    }

    /// Activate the next queued project(s) if slots are available
    fn activate_next_queued(&mut self) {
        let active_count = self.active_research().len();
        let max_slots = if self.config.allow_parallel_research {
            self.config.max_parallel_slots
        } else {
            1
        };

        let available_slots = max_slots.saturating_sub(active_count);

        for _ in 0..available_slots {
            if let Some(next_id) = self
                .queue
                .iter()
                .find(|id| {
                    self.projects
                        .get(id)
                        .map(|p| p.status == ResearchStatus::Queued)
                        .unwrap_or(false)
                })
                .cloned()
            {
                if let Some(project) = self.projects.get_mut(&next_id) {
                    project.status = ResearchStatus::InProgress;
                }
            } else {
                break;
            }
        }
    }

    /// Complete a research project (manually, or via event)
    pub fn complete(&mut self, id: &ResearchId) -> Result<ResearchResult, ResearchError> {
        // Check status and get data before mutation
        let (metrics, metadata) = {
            let project = self.projects.get(id).ok_or(ResearchError::NotFound)?;

            if project.status != ResearchStatus::InProgress {
                return Err(ResearchError::NotInProgress);
            }

            (project.metrics.clone(), project.metadata.clone())
        };

        // Now mutate
        if let Some(project) = self.projects.get_mut(id) {
            project.status = ResearchStatus::Completed;
            project.progress = 1.0;
        }

        // Remove from queue
        self.queue.retain(|qid| qid != id);

        // Activate next queued
        self.activate_next_queued();

        Ok(ResearchResult {
            project_id: id.clone(),
            success: true,
            final_metrics: metrics,
            metadata,
        })
    }

    /// Cancel a research project
    pub fn cancel(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let project = self
            .projects
            .get_mut(id)
            .ok_or(ResearchError::NotFound)?;

        if project.status == ResearchStatus::Completed {
            return Err(ResearchError::AlreadyCompleted);
        }

        project.status = ResearchStatus::Available;
        project.progress = 0.0;

        // Remove from queue
        self.queue.retain(|qid| qid != id);

        // Activate next queued
        self.activate_next_queued();

        Ok(())
    }

    /// List all available research projects (not completed, not queued)
    pub fn available_research(&self) -> Vec<&ResearchProject> {
        self.projects
            .values()
            .filter(|p| p.status == ResearchStatus::Available)
            .collect()
    }

    /// List all completed research projects
    pub fn completed_research(&self) -> Vec<&ResearchProject> {
        self.projects
            .values()
            .filter(|p| p.status == ResearchStatus::Completed)
            .collect()
    }

    /// Get all projects (for iteration)
    pub fn iter(&self) -> impl Iterator<Item = &ResearchProject> {
        self.projects.values()
    }

    /// Get the current queue order
    pub fn queue_order(&self) -> &[ResearchId] {
        &self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project(id: &str, name: &str) -> ResearchProject {
        ResearchProject::new(id, name, "Test description")
    }

    #[test]
    fn test_registry_creation() {
        let registry = ResearchRegistry::new();
        assert_eq!(registry.config.allow_parallel_research, false);
        assert_eq!(registry.config.max_parallel_slots, 1);
        assert_eq!(registry.config.base_progress_per_turn, 0.1);
    }

    #[test]
    fn test_define_project() {
        let mut registry = ResearchRegistry::new();
        let project = create_test_project("test", "Test Project");

        registry.define(project);

        assert!(registry.get(&ResearchId::new("test")).is_some());
    }

    #[test]
    fn test_queue_project() {
        let mut registry = ResearchRegistry::new();
        let project = create_test_project("test", "Test Project");
        registry.define(project);

        let result = registry.queue(&ResearchId::new("test"));
        assert!(result.is_ok());

        let queued_project = registry.get(&ResearchId::new("test")).unwrap();
        // Should be InProgress (auto-activated since single queue)
        assert_eq!(queued_project.status, ResearchStatus::InProgress);
    }

    #[test]
    fn test_queue_nonexistent_project() {
        let mut registry = ResearchRegistry::new();
        let result = registry.queue(&ResearchId::new("nonexistent"));
        assert!(matches!(result, Err(ResearchError::NotFound)));
    }

    #[test]
    fn test_queue_already_queued() {
        let mut registry = ResearchRegistry::new();
        let project = create_test_project("test", "Test Project");
        registry.define(project);

        registry.queue(&ResearchId::new("test")).unwrap();
        let result = registry.queue(&ResearchId::new("test"));
        assert!(matches!(result, Err(ResearchError::AlreadyQueued)));
    }

    #[test]
    fn test_advance_progress() {
        let mut registry = ResearchRegistry::new();
        let project = create_test_project("test", "Test Project");
        registry.define(project);

        registry.queue(&ResearchId::new("test")).unwrap();

        // Advance 50%
        let completed = registry.advance_progress(0.5);
        assert!(completed.is_empty());

        let project = registry.get(&ResearchId::new("test")).unwrap();
        assert_eq!(project.progress, 0.5);
        assert_eq!(project.status, ResearchStatus::InProgress);

        // Advance to completion
        let completed = registry.advance_progress(0.5);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].as_str(), "test");

        let project = registry.get(&ResearchId::new("test")).unwrap();
        assert_eq!(project.progress, 1.0);
        assert_eq!(project.status, ResearchStatus::Completed);
    }

    #[test]
    fn test_parallel_research() {
        let mut registry = ResearchRegistry::with_config(ResearchConfig {
            allow_parallel_research: true,
            max_parallel_slots: 2,
            ..Default::default()
        });

        let project1 = create_test_project("test1", "Test 1");
        let project2 = create_test_project("test2", "Test 2");
        let project3 = create_test_project("test3", "Test 3");

        registry.define(project1);
        registry.define(project2);
        registry.define(project3);

        registry.queue(&ResearchId::new("test1")).unwrap();
        registry.queue(&ResearchId::new("test2")).unwrap();
        registry.queue(&ResearchId::new("test3")).unwrap();

        // First two should be InProgress
        let active = registry.active_research();
        assert_eq!(active.len(), 2);

        // Third should be Queued
        let queued = registry.queued_research();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].id.as_str(), "test3");
    }

    #[test]
    fn test_complete_project() {
        let mut registry = ResearchRegistry::new();
        let project = create_test_project("test", "Test Project");
        registry.define(project);

        registry.queue(&ResearchId::new("test")).unwrap();

        let result = registry.complete(&ResearchId::new("test"));
        assert!(result.is_ok());

        let project = registry.get(&ResearchId::new("test")).unwrap();
        assert_eq!(project.status, ResearchStatus::Completed);
    }

    #[test]
    fn test_cancel_project() {
        let mut registry = ResearchRegistry::new();
        let project = create_test_project("test", "Test Project");
        registry.define(project);

        registry.queue(&ResearchId::new("test")).unwrap();

        let result = registry.cancel(&ResearchId::new("test"));
        assert!(result.is_ok());

        let project = registry.get(&ResearchId::new("test")).unwrap();
        assert_eq!(project.status, ResearchStatus::Available);
        assert_eq!(project.progress, 0.0);
    }

    #[test]
    fn test_available_research() {
        let mut registry = ResearchRegistry::new();
        let project1 = create_test_project("test1", "Test 1");
        let project2 = create_test_project("test2", "Test 2");

        registry.define(project1);
        registry.define(project2);

        registry.queue(&ResearchId::new("test1")).unwrap();

        let available = registry.available_research();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id.as_str(), "test2");
    }

    #[test]
    fn test_retry_failed_project() {
        let mut registry = ResearchRegistry::new();
        let mut project = create_test_project("test", "Test Project");
        project.status = ResearchStatus::Failed;
        project.progress = 0.5;

        registry.define(project);

        // Should allow retrying
        let result = registry.queue(&ResearchId::new("test"));
        assert!(result.is_ok());

        let project = registry.get(&ResearchId::new("test")).unwrap();
        assert_eq!(project.status, ResearchStatus::InProgress); // Auto-activated
        assert_eq!(project.progress, 0.0); // Reset
    }
}
