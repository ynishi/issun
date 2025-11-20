//! Hook trait for custom research behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::*;

/// Trait for custom research behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., progress modifiers based on game state)
/// - Direct resource modification (e.g., unlocking units, applying bonuses)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait ResearchHook: Send + Sync {
    /// Called when a research project is queued
    ///
    /// This is called immediately after the project is queued in the registry,
    /// allowing you to modify other resources (e.g., deduct costs, log events).
    ///
    /// # Arguments
    ///
    /// * `project` - The project being queued
    /// * `resources` - Access to game resources for modification
    async fn on_research_queued(
        &self,
        _project: &ResearchProject,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when a research project starts (moves from Queued to InProgress)
    ///
    /// # Arguments
    ///
    /// * `project` - The project starting
    /// * `resources` - Access to game resources for modification
    async fn on_research_started(
        &self,
        _project: &ResearchProject,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate research cost and validate
    ///
    /// Return `Ok(cost)` to allow queuing, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to validate
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// `Ok(cost)` if queuing is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Returns project's base cost (or 0 if not set)
    async fn calculate_research_cost(
        &self,
        project: &ResearchProject,
        _resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Default: use base cost
        Ok(project.cost)
    }

    /// Calculate research progress per turn/tick
    ///
    /// Allows game-specific bonuses/penalties based on context.
    ///
    /// # Arguments
    ///
    /// * `project` - The project in progress
    /// * `base_progress` - Base progress from config
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Effective progress amount (potentially modified by game state)
    ///
    /// # Default
    ///
    /// Returns base progress unchanged
    async fn calculate_progress(
        &self,
        _project: &ResearchProject,
        base_progress: f32,
        _resources: &ResourceContext,
    ) -> f32 {
        // Default: no modification
        base_progress
    }

    /// Called when research is completed
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets the ResearchResult and updates other resources.
    /// For example:
    /// - Strategy game: Unlock units, apply tech bonuses
    /// - RPG: Learn skills, unlock abilities
    /// - Crafting: Unlock recipes, improve quality
    ///
    /// # Arguments
    ///
    /// * `project` - The completed project
    /// * `result` - Result data (success/failure, metrics, metadata)
    /// * `resources` - Access to game resources for modification
    async fn on_research_completed(
        &self,
        _project: &ResearchProject,
        _result: &ResearchResult,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when research fails or is cancelled
    ///
    /// Can modify other resources based on failure (e.g., partial refund).
    ///
    /// # Arguments
    ///
    /// * `project` - The failed project
    /// * `resources` - Access to game resources for modification
    async fn on_research_failed(
        &self,
        _project: &ResearchProject,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Validate whether a project can be queued
    ///
    /// Return `Ok(())` to allow queuing, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to validate
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if queuing is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows queuing
    async fn validate_prerequisites(
        &self,
        _project: &ResearchProject,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultResearchHook;

#[async_trait]
impl ResearchHook for DefaultResearchHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultResearchHook;
        let project = ResearchProject::new("test", "Test", "Test");
        let result = ResearchResult::new(ResearchId::new("test"), true);
        let mut resources = ResourceContext::new();

        // Should not panic
        hook.on_research_queued(&project, &mut resources).await;
        hook.on_research_started(&project, &mut resources).await;

        let cost = hook
            .calculate_research_cost(&project, &resources)
            .await;
        assert_eq!(cost, Ok(0));

        let progress = hook.calculate_progress(&project, 0.1, &resources).await;
        assert_eq!(progress, 0.1);

        hook.on_research_completed(&project, &result, &mut resources)
            .await;
        hook.on_research_failed(&project, &mut resources).await;

        let validation = hook.validate_prerequisites(&project, &resources).await;
        assert!(validation.is_ok());
    }

    #[tokio::test]
    async fn test_default_hook_returns_project_cost() {
        let hook = DefaultResearchHook;
        let project = ResearchProject::new("test", "Test", "Test").with_cost(5000);
        let resources = ResourceContext::new();

        let cost = hook
            .calculate_research_cost(&project, &resources)
            .await;
        assert_eq!(cost, Ok(5000));
    }
}
