//! Hook trait for game-specific generation behavior (ECS version)

use super::state_ecs::GenerationStateECS;
use async_trait::async_trait;

/// Hook for customizing generation behavior (ECS version)
#[async_trait]
pub trait GenerationHookECS: Send + Sync {
    /// Called when generation status changes
    ///
    /// # Arguments
    /// * `entity` - Entity that changed status
    /// * `new_progress` - New generation progress value
    async fn on_generation_status_changed(&self, entity: hecs::Entity, new_progress: f32) {
        let _ = (entity, new_progress);
        // Default: no-op
    }

    /// Called when generation completes (reaches 100%)
    ///
    /// # Arguments
    /// * `entity` - Entity that completed generation
    /// * `state` - Current ECS state (for querying components)
    async fn on_generation_completed(&self, entity: hecs::Entity, state: &GenerationStateECS) {
        let _ = (entity, state);
        // Default: no-op
    }

    /// Check if entity should generate this tick
    ///
    /// # Arguments
    /// * `entity` - Entity to check
    /// * `state` - Current ECS state
    ///
    /// # Returns
    /// true if generation should proceed
    async fn should_generate(&self, entity: hecs::Entity, state: &GenerationStateECS) -> bool {
        let _ = (entity, state);
        // Default: always generate
        true
    }

    /// Calculate resource consumption for generation
    ///
    /// # Arguments
    /// * `entity` - Entity generating
    /// * `progress_amount` - Amount of progress being added
    ///
    /// # Returns
    /// Resources consumed (resource_id, amount)
    async fn calculate_resource_consumption(
        &self,
        entity: hecs::Entity,
        progress_amount: f32,
    ) -> Vec<(String, u32)> {
        let _ = (entity, progress_amount);
        // Default: no resources consumed
        Vec::new()
    }

    /// Modify generation rate before calculation
    ///
    /// # Arguments
    /// * `entity` - Entity about to generate
    /// * `base_rate` - Calculated base generation rate
    ///
    /// # Returns
    /// Modified generation rate
    async fn modify_generation_rate(&self, entity: hecs::Entity, base_rate: f32) -> f32 {
        let _ = entity;
        base_rate
    }

    /// Called when generation is paused
    ///
    /// # Arguments
    /// * `entity` - Entity that was paused
    async fn on_generation_paused(&self, entity: hecs::Entity) {
        let _ = entity;
        // Default: no-op
    }

    /// Called when generation is resumed
    ///
    /// # Arguments
    /// * `entity` - Entity that was resumed
    async fn on_generation_resumed(&self, entity: hecs::Entity) {
        let _ = entity;
        // Default: no-op
    }
}

/// Default implementation (no-op)
pub struct DefaultGenerationHookECS;

#[async_trait]
impl GenerationHookECS for DefaultGenerationHookECS {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestHook {
        on_completed_called: Arc<AtomicBool>,
        should_generate_result: bool,
        resource_multiplier: f32,
    }

    #[async_trait]
    impl GenerationHookECS for TestHook {
        async fn on_generation_completed(
            &self,
            _entity: hecs::Entity,
            _state: &GenerationStateECS,
        ) {
            self.on_completed_called.store(true, Ordering::SeqCst);
        }

        async fn should_generate(
            &self,
            _entity: hecs::Entity,
            _state: &GenerationStateECS,
        ) -> bool {
            self.should_generate_result
        }

        async fn calculate_resource_consumption(
            &self,
            _entity: hecs::Entity,
            progress_amount: f32,
        ) -> Vec<(String, u32)> {
            vec![(
                "wood".to_string(),
                (progress_amount * self.resource_multiplier) as u32,
            )]
        }

        async fn modify_generation_rate(&self, _entity: hecs::Entity, base_rate: f32) -> f32 {
            base_rate * 2.0 // Double rate
        }
    }

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultGenerationHookECS;
        let state = GenerationStateECS::new();
        let entity = hecs::Entity::DANGLING;

        // Should not panic
        hook.on_generation_status_changed(entity, 50.0).await;
        hook.on_generation_completed(entity, &state).await;

        let should_gen = hook.should_generate(entity, &state).await;
        assert!(should_gen);

        let resources = hook.calculate_resource_consumption(entity, 10.0).await;
        assert!(resources.is_empty());

        let rate = hook.modify_generation_rate(entity, 5.0).await;
        assert_eq!(rate, 5.0);

        hook.on_generation_paused(entity).await;
        hook.on_generation_resumed(entity).await;
    }

    #[tokio::test]
    async fn test_custom_hook() {
        let called = Arc::new(AtomicBool::new(false));
        let hook = TestHook {
            on_completed_called: called.clone(),
            should_generate_result: false,
            resource_multiplier: 2.0,
        };

        let state = GenerationStateECS::new();
        let entity = hecs::Entity::DANGLING;

        // Test on_generation_completed
        hook.on_generation_completed(entity, &state).await;
        assert!(called.load(Ordering::SeqCst));

        // Test should_generate
        let should_gen = hook.should_generate(entity, &state).await;
        assert!(!should_gen);

        // Test resource consumption
        let resources = hook.calculate_resource_consumption(entity, 10.0).await;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].0, "wood");
        assert_eq!(resources[0].1, 20); // 10 * 2.0

        // Test rate modification
        let rate = hook.modify_generation_rate(entity, 5.0).await;
        assert_eq!(rate, 10.0); // 5 * 2.0
    }

    #[tokio::test]
    async fn test_hook_with_state_query() {
        struct QueryHook {
            blocked_count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl GenerationHookECS for QueryHook {
            async fn should_generate(
                &self,
                entity: hecs::Entity,
                state: &GenerationStateECS,
            ) -> bool {
                // Check if entity has generation component and is not paused
                if let Ok(generation) = state.world.get::<&super::super::types::Generation>(entity)
                {
                    if generation.paused {
                        self.blocked_count.fetch_add(1, Ordering::SeqCst);
                        return false;
                    }
                }
                true
            }
        }

        let mut state = GenerationStateECS::new();
        let blocked_count = Arc::new(AtomicUsize::new(0));
        let hook = QueryHook {
            blocked_count: blocked_count.clone(),
        };

        // Spawn paused entity
        let mut gen = super::super::types::Generation::new(
            100.0,
            1.0,
            super::super::types::GenerationType::Organic,
        );
        gen.pause();

        let entity = state.spawn_entity(gen, super::super::types::GenerationEnvironment::default());

        // Check should_generate
        let should_gen = hook.should_generate(entity, &state).await;
        assert!(!should_gen);
        assert_eq!(blocked_count.load(Ordering::SeqCst), 1);
    }
}
