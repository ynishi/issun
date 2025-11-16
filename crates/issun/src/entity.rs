//! Entity system for ISSUN
//!
//! Entities represent game objects (Player, Enemy, Item, etc.)

use async_trait::async_trait;
use crate::context::Context;

/// Entity trait for game objects
///
/// Entities are the primary game objects that can be updated and managed.
/// Examples: Player, Enemy, NPC, Item
#[async_trait]
pub trait Entity: Send + Sync {
    /// Unique identifier for this entity
    fn id(&self) -> &str;

    /// Update the entity state
    ///
    /// Called each frame/turn to update entity logic
    async fn update(&mut self, ctx: &mut Context);

    /// Optional: Called when entity is spawned
    async fn on_spawn(&mut self, _ctx: &mut Context) {}

    /// Optional: Called when entity is destroyed
    async fn on_destroy(&mut self, _ctx: &mut Context) {}

    /// Optional: Check if entity should be removed
    fn is_dead(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity {
        id: String,
        update_count: usize,
    }

    #[async_trait]
    impl Entity for TestEntity {
        fn id(&self) -> &str {
            &self.id
        }

        async fn update(&mut self, _ctx: &mut Context) {
            self.update_count += 1;
        }

        fn is_dead(&self) -> bool {
            self.update_count > 10
        }
    }

    #[tokio::test]
    async fn test_entity_creation() {
        let entity = TestEntity {
            id: "test_entity".to_string(),
            update_count: 0,
        };

        assert_eq!(entity.id(), "test_entity");
        assert!(!entity.is_dead());
    }

    #[tokio::test]
    async fn test_entity_update() {
        let mut entity = TestEntity {
            id: "test_entity".to_string(),
            update_count: 0,
        };
        let mut ctx = Context::new();

        entity.update(&mut ctx).await;
        assert_eq!(entity.update_count, 1);

        for _ in 0..10 {
            entity.update(&mut ctx).await;
        }
        assert!(entity.is_dead());
    }
}
