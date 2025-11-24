//! Custom entropy hook for plant decay

use async_trait::async_trait;
use hecs::Entity;
use issun::plugin::entropy::{EntropyHookECS, EntropyStateECS};

/// Garden-specific entropy hook
pub struct GardenEntropyHook;

#[async_trait]
impl EntropyHookECS for GardenEntropyHook {
    async fn on_durability_status_changed(&self, _entity: Entity, new_durability: f32) {
        // Log health changes
        if new_durability <= 20.0 {
            println!("⚠️ Plant is dying (health: {:.1}%)", new_durability);
        }
    }

    async fn on_entity_destroyed(&self, entity: Entity, _state: &EntropyStateECS) {
        // Plant has died
        println!("💀 Plant {:?} has died!", entity);
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
