//! Hook trait for custom territory behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::*;

/// Trait for custom territory behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., development cost)
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
/// use issun::plugin::territory::{TerritoryHook, Territory, ControlChanged};
/// use issun::context::ResourceContext;
/// use async_trait::async_trait;
///
/// struct GameLogHook;
///
/// #[async_trait]
/// impl TerritoryHook for GameLogHook {
///     async fn on_control_changed(
///         &self,
///         territory: &Territory,
///         change: &ControlChanged,
///         resources: &mut ResourceContext,
///     ) {
///         if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
///             ctx.record(format!(
///                 "{} control: {:.0}% → {:.0}%",
///                 territory.name,
///                 change.old_control * 100.0,
///                 change.new_control * 100.0
///             ));
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait TerritoryHook: Send + Sync {
    /// Called when territory control changes
    ///
    /// This is called immediately after control changes, allowing you to
    /// modify other resources (e.g., update player influence, log events).
    ///
    /// # Arguments
    ///
    /// * `territory` - The territory that changed
    /// * `change` - Details about the control change
    /// * `resources` - Access to game resources for modification
    async fn on_control_changed(
        &self,
        _territory: &Territory,
        _change: &ControlChanged,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate development cost and validate
    ///
    /// Return `Ok(cost)` to allow development, `Err` to prevent.
    /// This is synchronous because the caller needs the result immediately.
    ///
    /// # Arguments
    ///
    /// * `territory` - The territory to develop (definition only)
    /// * `current_level` - Current development level from TerritoryState
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// `Ok(cost)` if development is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Returns fixed cost based on development level: `100 * (level + 1)`
    async fn calculate_development_cost(
        &self,
        _territory: &Territory,
        current_level: u32,
        _resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Default: fixed cost based on level
        Ok(100 * (current_level + 1) as i64)
    }

    /// Called after territory is developed
    ///
    /// Can modify other resources based on development.
    ///
    /// # Arguments
    ///
    /// * `territory` - The territory that was developed
    /// * `developed` - Details about the development
    /// * `resources` - Access to game resources for modification
    async fn on_developed(
        &self,
        _territory: &Territory,
        _developed: &Developed,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate final effects for a territory
    ///
    /// Allows game-specific calculations (e.g., bonuses from policies, neighbors, etc.)
    ///
    /// # Arguments
    ///
    /// * `territory` - The territory to calculate effects for
    /// * `base_effects` - Base effects from territory
    /// * `resources` - Access to game resources for calculations
    ///
    /// # Returns
    ///
    /// Final effects with any game-specific modifiers applied
    ///
    /// # Default
    ///
    /// Returns base effects unchanged
    async fn calculate_effects(
        &self,
        _territory: &Territory,
        base_effects: TerritoryEffects,
        _resources: &ResourceContext,
    ) -> TerritoryEffects {
        // Default: return base effects
        base_effects
    }
}

/// No-op default hook
///
/// This is the default hook used when no custom hook is provided.
/// It implements all hook methods with default behavior.
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultTerritoryHook;

#[async_trait]
impl TerritoryHook for DefaultTerritoryHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultTerritoryHook;
        let mut resources = ResourceContext::new();

        let territory = Territory::new("test", "Test Territory");
        let change = ControlChanged {
            id: TerritoryId::new("test"),
            old_control: 0.5,
            new_control: 0.7,
            delta: 0.2,
        };

        // Should not panic
        hook.on_control_changed(&territory, &change, &mut resources)
            .await;

        let cost = hook
            .calculate_development_cost(&territory, 0, &resources)
            .await;
        assert!(cost.is_ok());
        assert_eq!(cost.unwrap(), 100); // (0 + 1) * 100

        let developed = Developed {
            id: TerritoryId::new("test"),
            old_level: 0,
            new_level: 1,
        };
        hook.on_developed(&territory, &developed, &mut resources)
            .await;

        let effects = TerritoryEffects::default();
        let result = hook
            .calculate_effects(&territory, effects.clone(), &resources)
            .await;
        assert_eq!(result.income_multiplier, effects.income_multiplier);
    }

    #[tokio::test]
    async fn test_default_development_cost_scaling() {
        let hook = DefaultTerritoryHook;
        let resources = ResourceContext::new();

        let territory = Territory::new("test", "Test");

        // Level 0 -> 1
        let cost = hook
            .calculate_development_cost(&territory, 0, &resources)
            .await
            .unwrap();
        assert_eq!(cost, 100); // (0 + 1) * 100

        // Level 3 -> 4
        let cost = hook
            .calculate_development_cost(&territory, 3, &resources)
            .await
            .unwrap();
        assert_eq!(cost, 400); // (3 + 1) * 100
    }
}
