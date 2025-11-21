//! Research runtime state (Mutable)

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime state for a single research project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    /// Current status
    pub status: ResearchStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            status: ResearchStatus::Available,
            progress: 0.0,
        }
    }
}

impl Default for ProjectState {
    fn default() -> Self {
        Self::new()
    }
}

/// Research runtime state (Mutable)
///
/// Contains research queue and project runtime state that changes during gameplay.
/// This is a save/load target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchState {
    /// Current research queue (ordered list of project IDs)
    queue: Vec<ResearchId>,

    /// Runtime state for each project (status, progress)
    project_states: HashMap<ResearchId, ProjectState>,
}

impl State for ResearchState {}

impl ResearchState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            project_states: HashMap::new(),
        }
    }

    // ========================================
    // Project State Management
    // ========================================

    /// Get project state (or default if not exists)
    pub fn get_state(&self, id: &ResearchId) -> ProjectState {
        self.project_states.get(id).cloned().unwrap_or_default()
    }

    /// Get project status
    pub fn get_status(&self, id: &ResearchId) -> ResearchStatus {
        self.get_state(id).status
    }

    /// Get project progress
    pub fn get_progress(&self, id: &ResearchId) -> f32 {
        self.get_state(id).progress
    }

    /// Set project status
    pub fn set_status(&mut self, id: &ResearchId, status: ResearchStatus) {
        self.project_states.entry(id.clone()).or_default().status = status;
    }

    /// Set project progress
    pub fn set_progress(&mut self, id: &ResearchId, progress: f32) {
        self.project_states.entry(id.clone()).or_default().progress = progress.clamp(0.0, 1.0);
    }

    /// Add progress to a project
    pub fn add_progress(&mut self, id: &ResearchId, amount: f32) {
        let current = self.get_progress(id);
        self.set_progress(id, current + amount);
    }

    /// Reset project state to available
    pub fn reset(&mut self, id: &ResearchId) {
        self.set_status(id, ResearchStatus::Available);
        self.set_progress(id, 0.0);
    }

    // ========================================
    // Queue Management
    // ========================================

    /// Queue a project for research
    ///
    /// Returns `Ok(())` if queued successfully,
    /// `Err(ResearchError)` if already queued/completed.
    pub fn queue(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let status = self.get_status(id);

        match status {
            ResearchStatus::Available | ResearchStatus::Failed => {
                self.set_status(id, ResearchStatus::Queued);
                self.set_progress(id, 0.0); // Reset progress on retry
                self.queue.push(id.clone());
                Ok(())
            }
            ResearchStatus::Queued | ResearchStatus::InProgress => {
                Err(ResearchError::AlreadyQueued)
            }
            ResearchStatus::Completed => Err(ResearchError::AlreadyCompleted),
        }
    }

    /// Get the current queue order
    pub fn queue_order(&self) -> &[ResearchId] {
        &self.queue
    }

    /// Remove a project from the queue
    pub fn remove_from_queue(&mut self, id: &ResearchId) {
        self.queue.retain(|qid| qid != id);
    }

    /// Get all queued projects (with Queued status)
    pub fn queued_projects(&self) -> Vec<ResearchId> {
        self.queue
            .iter()
            .filter(|id| self.get_status(id) == ResearchStatus::Queued)
            .cloned()
            .collect()
    }

    /// Get all active projects (with InProgress status)
    pub fn active_projects(&self) -> Vec<ResearchId> {
        self.project_states
            .iter()
            .filter(|(_, state)| state.status == ResearchStatus::InProgress)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Activate the next queued project
    ///
    /// Changes the first queued project to InProgress.
    /// Returns the activated project ID, or None if no projects available.
    pub fn activate_next(&mut self) -> Option<ResearchId> {
        let next_id = self
            .queue
            .iter()
            .find(|id| self.get_status(id) == ResearchStatus::Queued)
            .cloned()?;

        self.set_status(&next_id, ResearchStatus::InProgress);
        Some(next_id)
    }

    /// Activate next queued projects up to max_slots
    ///
    /// Returns the number of projects activated.
    pub fn activate_next_queued(&mut self, max_active: usize) -> usize {
        let current_active = self.active_projects().len();
        let available_slots = max_active.saturating_sub(current_active);

        let mut activated = 0;
        for _ in 0..available_slots {
            if self.activate_next().is_some() {
                activated += 1;
            } else {
                break;
            }
        }

        activated
    }

    /// Complete a project
    pub fn complete(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let status = self.get_status(id);

        if status != ResearchStatus::InProgress {
            return Err(ResearchError::NotInProgress);
        }

        self.set_status(id, ResearchStatus::Completed);
        self.set_progress(id, 1.0);
        self.remove_from_queue(id);

        Ok(())
    }

    /// Fail a project
    pub fn fail(&mut self, id: &ResearchId) {
        self.set_status(id, ResearchStatus::Failed);
        self.remove_from_queue(id);
    }

    /// Cancel a project
    pub fn cancel(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let status = self.get_status(id);

        if status == ResearchStatus::Completed {
            return Err(ResearchError::AlreadyCompleted);
        }

        self.reset(id);
        self.remove_from_queue(id);

        Ok(())
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.queue.clear();
        self.project_states.clear();
    }
}

impl Default for ResearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = ResearchState::new();
        assert!(state.queue.is_empty());
        assert!(state.project_states.is_empty());
    }

    #[test]
    fn test_get_state_default() {
        let state = ResearchState::new();
        let project_state = state.get_state(&ResearchId::new("test"));
        assert_eq!(project_state.status, ResearchStatus::Available);
        assert_eq!(project_state.progress, 0.0);
    }

    #[test]
    fn test_set_and_get_status() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.set_status(&id, ResearchStatus::InProgress);
        assert_eq!(state.get_status(&id), ResearchStatus::InProgress);
    }

    #[test]
    fn test_set_and_get_progress() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.set_progress(&id, 0.5);
        assert_eq!(state.get_progress(&id), 0.5);

        // Test clamping
        state.set_progress(&id, 1.5);
        assert_eq!(state.get_progress(&id), 1.0);

        state.set_progress(&id, -0.5);
        assert_eq!(state.get_progress(&id), 0.0);
    }

    #[test]
    fn test_add_progress() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.add_progress(&id, 0.3);
        assert_eq!(state.get_progress(&id), 0.3);

        state.add_progress(&id, 0.3);
        assert_eq!(state.get_progress(&id), 0.6);
    }

    #[test]
    fn test_queue_project() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        let result = state.queue(&id);
        assert!(result.is_ok());
        assert_eq!(state.get_status(&id), ResearchStatus::Queued);
        assert_eq!(state.queue.len(), 1);
    }

    #[test]
    fn test_queue_already_queued() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.queue(&id).unwrap();
        let result = state.queue(&id);
        assert!(matches!(result, Err(ResearchError::AlreadyQueued)));
    }

    #[test]
    fn test_activate_next() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.queue(&id).unwrap();

        let activated = state.activate_next();
        assert_eq!(activated, Some(id.clone()));
        assert_eq!(state.get_status(&id), ResearchStatus::InProgress);
    }

    #[test]
    fn test_activate_next_queued() {
        let mut state = ResearchState::new();
        let id1 = ResearchId::new("test1");
        let id2 = ResearchId::new("test2");
        let id3 = ResearchId::new("test3");

        state.queue(&id1).unwrap();
        state.queue(&id2).unwrap();
        state.queue(&id3).unwrap();

        let activated = state.activate_next_queued(2);
        assert_eq!(activated, 2);

        assert_eq!(state.get_status(&id1), ResearchStatus::InProgress);
        assert_eq!(state.get_status(&id2), ResearchStatus::InProgress);
        assert_eq!(state.get_status(&id3), ResearchStatus::Queued);
    }

    #[test]
    fn test_complete_project() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.queue(&id).unwrap();
        state.activate_next();

        let result = state.complete(&id);
        assert!(result.is_ok());
        assert_eq!(state.get_status(&id), ResearchStatus::Completed);
        assert_eq!(state.get_progress(&id), 1.0);
        assert!(!state.queue.contains(&id));
    }

    #[test]
    fn test_cancel_project() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.queue(&id).unwrap();
        state.activate_next();

        let result = state.cancel(&id);
        assert!(result.is_ok());
        assert_eq!(state.get_status(&id), ResearchStatus::Available);
        assert_eq!(state.get_progress(&id), 0.0);
        assert!(!state.queue.contains(&id));
    }

    #[test]
    fn test_fail_project() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.queue(&id).unwrap();
        state.activate_next();

        state.fail(&id);
        assert_eq!(state.get_status(&id), ResearchStatus::Failed);
        assert!(!state.queue.contains(&id));
    }

    #[test]
    fn test_retry_failed_project() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.set_status(&id, ResearchStatus::Failed);
        state.set_progress(&id, 0.5);

        let result = state.queue(&id);
        assert!(result.is_ok());
        assert_eq!(state.get_status(&id), ResearchStatus::Queued);
        assert_eq!(state.get_progress(&id), 0.0); // Reset on retry
    }

    #[test]
    fn test_clear() {
        let mut state = ResearchState::new();
        let id = ResearchId::new("test");

        state.queue(&id).unwrap();
        state.activate_next();

        state.clear();
        assert!(state.queue.is_empty());
        assert!(state.project_states.is_empty());
    }
}
