//! Hook trait for game-specific entropy behavior (ECS version)
//!
//! Uses hecs::Entity for entity identification.

use super::state_ecs::EntropyStateECS;
use async_trait::async_trait;

/// Hook for customizing entropy behavior (ECS version)
#[async_trait]
pub trait EntropyHookECS: Send + Sync {
    /// Called when entity durability status changes
    ///
    /// # Arguments
    /// * `entity` - Entity that changed status
    /// * `new_durability` - New durability value
    async fn on_durability_status_changed(&self, entity: hecs::Entity, new_durability: f32) {
        let _ = (entity, new_durability);
        // Default: no-op
    }

    /// Called when entity is destroyed (durability reaches 0)
    ///
    /// # Arguments
    /// * `entity` - Entity that was destroyed
    /// * `state` - Current ECS state (for querying components)
    async fn on_entity_destroyed(&self, entity: hecs::Entity, state: &EntropyStateECS) {
        let _ = (entity, state);
        // Default: no-op
    }

    /// Calculate repair cost for entity
    ///
    /// # Arguments
    /// * `entity` - Entity being repaired
    /// * `repair_amount` - Amount of durability being restored
    ///
    /// # Returns
    /// Cost of the repair operation
    async fn calculate_repair_cost(&self, entity: hecs::Entity, repair_amount: f32) -> f32 {
        let _ = entity;
        // Default: cost equals repair amount
        repair_amount
    }

    /// Called before decay update (can modify decay rate)
    ///
    /// # Arguments
    /// * `entity` - Entity about to decay
    /// * `base_decay` - Calculated base decay amount
    ///
    /// # Returns
    /// Modified decay amount
    async fn modify_decay(&self, entity: hecs::Entity, base_decay: f32) -> f32 {
        let _ = entity;
        base_decay
    }
}

/// Default implementation (no-op)
pub struct DefaultEntropyHookECS;

#[async_trait]
impl EntropyHookECS for DefaultEntropyHookECS {}

#[cfg(test)]
mod tests {

    use super::*;

    struct TestHook {
        on_destroyed_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait]
    impl EntropyHookECS for TestHook {
        async fn on_entity_destroyed(&self, _entity: hecs::Entity, _state: &EntropyStateECS) {
            self.on_destroyed_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }

        async fn calculate_repair_cost(&self, _entity: hecs::Entity, repair_amount: f32) -> f32 {
            repair_amount * 2.0 // Double cost
        }
    }

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultEntropyHookECS;
        let state = EntropyStateECS::new();
        let entity = hecs::Entity::DANGLING;

        // Should not panic
        hook.on_durability_status_changed(entity, 50.0).await;
        hook.on_entity_destroyed(entity, &state).await;

        let cost = hook.calculate_repair_cost(entity, 10.0).await;
        assert_eq!(cost, 10.0);

        let decay = hook.modify_decay(entity, 5.0).await;
        assert_eq!(decay, 5.0);
    }

    #[tokio::test]
    async fn test_custom_hook() {
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let hook = TestHook {
            on_destroyed_called: called.clone(),
        };

        let state = EntropyStateECS::new();
        let entity = hecs::Entity::DANGLING;

        hook.on_entity_destroyed(entity, &state).await;

        assert!(called.load(std::sync::atomic::Ordering::SeqCst));

        let cost = hook.calculate_repair_cost(entity, 10.0).await;
        assert_eq!(cost, 20.0); // Doubled
    }
}
