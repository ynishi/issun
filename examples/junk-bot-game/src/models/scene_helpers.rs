//! Scene helper functions
//!
//! Common logic used by multiple scene handlers

use super::entities::{generate_random_loot, Enemy, LootItem};
use super::{scenes::*, GameContext, GameScene};
use issun::prelude::{ResourceContext, SceneTransition};

/// Proceed to next floor in dungeon
pub async fn proceed_to_next_floor(resources: &mut ResourceContext) -> SceneTransition<GameScene> {
    let mut ctx = resources
        .get_mut::<GameContext>()
        .await
        .expect("GameContext resource not registered");
    let (advanced, current_floor, needs_floor4, next_room) = {
        if let Some(dungeon) = ctx.get_dungeon_mut() {
            // Advance to next floor
            let advanced = dungeon.advance();
            let current_floor = dungeon.current_floor_number();
            let needs_floor4 = dungeon.needs_floor4_selection();
            let next_room = dungeon.get_current_room().cloned();
            (advanced, current_floor, needs_floor4, next_room)
        } else {
            return SceneTransition::Switch(GameScene::Title(TitleSceneData::new()));
        }
    };

    if !advanced {
        // Dungeon complete! Victory!
        return SceneTransition::Switch(GameScene::Result(ResultSceneData::new(true, ctx.score)));
    }

    // Update floor number
    ctx.floor = current_floor as u32;

    // Check if we need floor 4 selection
    if needs_floor4 {
        return SceneTransition::Switch(GameScene::Floor4Choice(Floor4ChoiceSceneData::new()));
    }

    // Get next room and start combat
    if let Some(room) = next_room {
        return SceneTransition::Switch(GameScene::Combat(CombatSceneData::from_room(room)));
    }

    // Fallback: return to title
    SceneTransition::Switch(GameScene::Title(TitleSceneData::new()))
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
