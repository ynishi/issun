//! Custom generation hook for plant growth

use async_trait::async_trait;
use hecs::Entity;
use issun::plugin::generation::{GenerationHookECS, GenerationStateECS};

/// Garden-specific generation hook
pub struct GardenGenerationHook;

#[async_trait]
impl GenerationHookECS for GardenGenerationHook {
    async fn on_generation_status_changed(&self, _entity: hecs::Entity, new_progress: f32) {
        // Log growth milestones
        if new_progress >= 100.0 {
            // Plant is ready to harvest
        }
    }

    async fn on_generation_completed(&self, entity: Entity, _state: &GenerationStateECS) {
        // Plant has reached full growth
        println!("🌟 Plant {:?} is ready to harvest!", entity);
    }

    async fn should_generate(
        &self,
        entity: Entity,
        state: &GenerationStateECS,
    ) -> bool {
        // Check if plant is still alive (has durability)
        // In a real implementation, we would query Durability component
        // For now, always allow generation
        let _ = (entity, state);
        true
    }

    async fn calculate_resource_consumption(
        &self,
        _entity: hecs::Entity,
        progress_amount: f32,
    ) -> Vec<(String, u32)> {
        // Plants consume water to grow
        vec![("water".to_string(), (progress_amount * 0.1) as u32)]
    }

    async fn modify_generation_rate(&self, _entity: hecs::Entity, base_rate: f32) -> f32 {
        // Could apply weather effects, fertilizer bonuses, etc.
        base_rate
    }

    async fn on_generation_paused(&self, entity: Entity) {
        println!("⏸️ Plant {:?} growth paused", entity);
    }

    async fn on_generation_resumed(&self, entity: Entity) {
        println!("▶️ Plant {:?} growth resumed", entity);
    }
}
