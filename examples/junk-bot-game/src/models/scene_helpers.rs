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
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut drops = Vec::new();

    for _enemy in enemies.iter().filter(|e| !e.is_alive()) {
        // 30% base drop rate, apply multiplier
        let drop_rate = (0.3 * loot_multiplier).min(1.0);

        if rng.gen_bool(drop_rate as f64) {
            drops.push(generate_random_loot());
        }
    }

    drops
}
