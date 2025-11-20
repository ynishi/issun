//! Hook trait for custom faction behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::*;

/// Trait for custom faction behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., operation cost)
/// - Direct resource modification (e.g., logging to GameContext)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems
/// - Network replication (multiplayer)
/// - Audit log / replay
///
/// # Example
///
/// ```ignore
/// use issun::plugin::faction::{FactionHook, Faction, Operation, Outcome};
/// use issun::context::ResourceContext;
/// use async_trait::async_trait;
///
/// struct GameLogHook;
///
/// #[async_trait]
/// impl FactionHook for GameLogHook {
///     async fn on_operation_completed(
///         &self,
///         faction: &Faction,
///         operation: &Operation,
///         outcome: &Outcome,
///         resources: &mut ResourceContext,
///     ) {
///         if let Some(mut log) = resources.get_mut::<GameLog>().await {
///             log.record(format!(
///                 "{} completed {}: {}",
///                 faction.name,
///                 operation.name,
///                 if outcome.success { "SUCCESS" } else { "FAILED" }
///             ));
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait FactionHook: Send + Sync {
    /// Called when an operation is launched
    ///
    /// This is called immediately after the operation is added to the registry,
    /// allowing you to modify other resources (e.g., deduct budget, log events).
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction launching the operation
    /// * `operation` - The operation being launched
    /// * `resources` - Access to game resources for modification
    async fn on_operation_launched(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate operation cost and validate
    ///
    /// Return `Ok(cost)` to allow launch, `Err(reason)` to prevent.
    /// This is synchronous because the caller needs the result immediately.
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction launching the operation
    /// * `operation` - The operation to validate
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// `Ok(cost)` if launch is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Returns `Ok(0)` (free operations)
    async fn calculate_operation_cost(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Default: free operations
        Ok(0)
    }

    /// Called when an operation is completed
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets the Outcome (metrics/metadata) and updates other resources.
    /// For example:
    /// - Strategy game: Update territory control, casualties
    /// - RPG: Award XP, update quest progress
    /// - Sim: Update market share, revenue
    ///
    /// # Feedback Loop
    ///
    /// 1. `OperationResolveRequested` event published
    /// 2. Registry updates operation status
    /// 3. **This hook is called** (interpret outcome, update resources)
    /// 4. `OperationCompletedEvent` published (network replication)
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction that completed the operation
    /// * `operation` - The completed operation
    /// * `outcome` - Result data (success/failure, metrics, metadata)
    /// * `resources` - Access to game resources for modification
    async fn on_operation_completed(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _outcome: &Outcome,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when operation fails
    ///
    /// Can modify other resources based on failure (e.g., reputation penalty).
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction whose operation failed
    /// * `operation` - The failed operation
    /// * `resources` - Access to game resources for modification
    async fn on_operation_failed(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// No-op default hook
///
/// This is the default hook used when no custom hook is provided.
/// It implements all hook methods with default behavior.
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultFactionHook;

#[async_trait]
impl FactionHook for DefaultFactionHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultFactionHook;
        let mut resources = ResourceContext::new();

        let faction = Faction::new("crimson", "Crimson Syndicate");
        let operation = Operation::new("op-001", FactionId::new("crimson"), "Test Operation");

        // Should not panic
        hook.on_operation_launched(&faction, &operation, &mut resources)
            .await;

        let cost = hook
            .calculate_operation_cost(&faction, &operation, &resources)
            .await;
        assert!(cost.is_ok());
        assert_eq!(cost.unwrap(), 0);

        let outcome = Outcome::new("op-001", true);
        hook.on_operation_completed(&faction, &operation, &outcome, &mut resources)
            .await;

        hook.on_operation_failed(&faction, &operation, &mut resources)
            .await;
    }
}
