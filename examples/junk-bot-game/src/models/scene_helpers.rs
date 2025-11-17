//! Scene helper functions
//!
//! Common logic used by multiple scene handlers

use super::{GameContext, GameScene, scenes::*};
use super::entities::{Enemy, LootItem, generate_random_loot};
use issun::prelude::SceneTransition;

/// Proceed to next floor in dungeon
pub fn proceed_to_next_floor(ctx: &mut GameContext) -> (GameScene, SceneTransition) {
    let (advanced, current_floor, needs_floor4, next_room) = {
        if let Some(dungeon) = ctx.get_dungeon_mut() {
            // Advance to next floor
            let advanced = dungeon.advance();
            let current_floor = dungeon.current_floor_number();
            let needs_floor4 = dungeon.needs_floor4_selection();
            let next_room = dungeon.get_current_room().cloned();
            (advanced, current_floor, needs_floor4, next_room)
        } else {
            return (GameScene::Title(TitleSceneData::new()), SceneTransition::Stay);
        }
    };

    if !advanced {
        // Dungeon complete! Victory!
        return (GameScene::Result(ResultSceneData::new(true, ctx.score)), SceneTransition::Stay);
    }

    // Update floor number
    ctx.floor = current_floor as u32;

    // Check if we need floor 4 selection
    if needs_floor4 {
        return (GameScene::Floor4Choice(Floor4ChoiceSceneData::new()), SceneTransition::Stay);
    }

    // Get next room and start combat
    if let Some(room) = next_room {
        return (GameScene::Combat(CombatSceneData::from_room(room)), SceneTransition::Stay);
    }

    // Fallback: return to title
    (GameScene::Title(TitleSceneData::new()), SceneTransition::Stay)
}

/// Generate loot drops from defeated enemies
pub fn generate_drops(enemies: &[Enemy], loot_multiplier: f32) -> Vec<LootItem> {
    use issun::prelude::{DropConfig, LootService};
    let mut rng = rand::thread_rng();

    // Use LootService for drop calculations
    let config = DropConfig::new(0.3, loot_multiplier); // 30% base rate
    let dead_enemy_count = enemies.iter().filter(|e| !e.is_alive()).count();
    let drop_count = LootService::calculate_drop_count(dead_enemy_count, &config, &mut rng);

    // Generate random loot for each drop
    (0..drop_count).map(|_| generate_random_loot()).collect()
}
