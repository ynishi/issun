//! Custom entropy hook for plant decay

use async_trait::async_trait;
use hecs::Entity;
use issun::plugin::entropy::{EntropyHookECS, EntropyStateECS};

/// Garden-specific entropy hook
pub struct GardenEntropyHook;

#[async_trait]
impl EntropyHookECS for GardenEntropyHook {
    async fn on_durability_status_changed(&self, _entity: Entity, _new_durability: f32) {
        // Log is now handled by Garden resource
    }

    async fn on_entity_destroyed(&self, _entity: Entity, _state: &EntropyStateECS) {
        // Log is now handled by Garden resource
    }

    async fn calculate_repair_cost(&self, _entity: Entity, repair_amount: f32) -> f32 {
        // Watering/fertilizing costs
        repair_amount * 0.5
    }

    async fn modify_decay(&self, _entity: Entity, base_decay: f32) -> f32 {
        // Could apply weather effects (drought increases decay, rain decreases)
        base_decay
    }
}
