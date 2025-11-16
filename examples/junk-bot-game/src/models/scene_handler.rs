//! Scene handler with Scene trait implementation

use super::{GameContext, GameScene, scenes::*};
use super::entities::{LootItem, ItemEffect, WeaponEffect, generate_random_loot, generate_random_cards, generate_random_rooms};
use issun::prelude::*;
use issun::ui::InputEvent;

/// Handle input for a scene and return next scene
pub fn handle_scene_input(
    scene: GameScene,
    ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    match scene {
        GameScene::Title(mut data) => {
            handle_title_input(&mut data, input, ctx)
        }
        GameScene::RoomSelection(mut data) => {
            handle_room_selection_input(&mut data, ctx, input)
        }
        GameScene::Combat(mut data) => {
            handle_combat_input(&mut data, ctx, input)
        }
        GameScene::DropCollection(mut data) => {
            handle_drop_collection_input(&mut data, ctx, input)
        }
        GameScene::CardSelection(mut data) => {
            handle_card_selection_input(&mut data, ctx, input)
        }
        GameScene::Floor4Choice(mut data) => {
            handle_floor4_choice_input(&mut data, ctx, input)
        }
        GameScene::Result(data) => {
            handle_result_input(data, ctx, input)
        }
    }
}

fn handle_title_input(
    data: &mut TitleSceneData,
    input: InputEvent,
    _ctx: &mut GameContext,
) -> (GameScene, SceneTransition) {
    match input {
        InputEvent::Cancel => {
            (GameScene::Title(data.clone()), SceneTransition::Quit)
        }
        InputEvent::Up => {
            if data.selected_index > 0 {
                data.selected_index -= 1;
            }
            (GameScene::Title(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Down => {
            if data.selected_index < 1 {
                data.selected_index += 1;
            }
            (GameScene::Title(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Select => {
            match data.selected_index {
                0 => {
                    // Start game - initialize dungeon
                    _ctx.start_dungeon();

                    // Get first room from dungeon
                    if let Some(dungeon) = _ctx.get_dungeon() {
                        if let Some(room) = dungeon.get_current_room() {
                            (GameScene::Combat(CombatSceneData::from_room(room.clone())), SceneTransition::Stay)
                        } else {
                            (GameScene::Title(data.clone()), SceneTransition::Stay)
                        }
                    } else {
                        (GameScene::Title(data.clone()), SceneTransition::Stay)
                    }
                }
                1 => {
                    // Quit
                    (GameScene::Title(data.clone()), SceneTransition::Quit)
                }
                _ => (GameScene::Title(data.clone()), SceneTransition::Stay)
            }
        }
        _ => (GameScene::Title(data.clone()), SceneTransition::Stay)
    }
}

fn handle_room_selection_input(
    data: &mut RoomSelectionSceneData,
    _ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    // Room selection is no longer used in dungeon mode, redirect to title
    match input {
        _ => (GameScene::Title(TitleSceneData::new()), SceneTransition::Stay)
    }
}

fn handle_combat_input(
    data: &mut CombatSceneData,
    ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    match input {
        InputEvent::Cancel => {
            (GameScene::Combat(data.clone()), SceneTransition::Quit)
        }
        InputEvent::Char('i') | InputEvent::Char('I') => {
            // Toggle inventory
            data.toggle_inventory();
            (GameScene::Combat(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Tab => {
            // Cycle equip target when inventory is shown
            if data.show_inventory {
                let bot_count = ctx.bots.iter().filter(|b| b.is_alive()).count();
                data.cycle_equip_target(bot_count);
            }
            (GameScene::Combat(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Up => {
            if data.show_inventory {
                data.move_inventory_up();
            }
            (GameScene::Combat(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Down => {
            if data.show_inventory {
                data.move_inventory_down(ctx.inventory.len());
            }
            (GameScene::Combat(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Select => {
            // Equip weapon when inventory is shown
            if data.show_inventory && !ctx.inventory.is_empty() {
                if data.inventory_cursor < ctx.inventory.len() {
                    let weapon = ctx.inventory[data.inventory_cursor].clone();
                    let target = data.equip_target.clone();
                    equip_weapon_to_target(ctx, &target, weapon, data);
                }
                data.show_inventory = false;
            }
            (GameScene::Combat(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Char(' ') => {
            // Process combat turn
            process_combat_turn(ctx, data);

            // Check win/lose conditions
            let all_enemies_dead = data.enemies.iter().all(|e| !e.is_alive());
            let party_dead = !ctx.is_party_alive();

            if all_enemies_dead {
                // Floor completed! Generate drops with room buff multiplier
                let drops = generate_drops(&data.enemies, data.room_buff.loot_multiplier());

                if drops.is_empty() {
                    // No drops, proceed to next floor logic
                    proceed_to_next_floor(ctx)
                } else {
                    // Show drop collection scene
                    (GameScene::DropCollection(DropCollectionSceneData::new(drops)), SceneTransition::Stay)
                }
            } else if party_dead {
                (GameScene::Result(ResultSceneData::new(false, ctx.score)), SceneTransition::Stay)
            } else {
                (GameScene::Combat(data.clone()), SceneTransition::Stay)
            }
        }
        _ => (GameScene::Combat(data.clone()), SceneTransition::Stay)
    }
}

fn handle_drop_collection_input(
    data: &mut DropCollectionSceneData,
    ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    match input {
        InputEvent::Up => {
            data.move_up();
            (GameScene::DropCollection(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Down => {
            data.move_down();
            (GameScene::DropCollection(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Select => {
            // Take selected item
            if let Some(item) = data.take_selected() {
                apply_loot_item(ctx, &item);
            }

            // If no more items, transition to card selection
            if !data.has_drops() {
                // Generate 3 random buff cards
                let cards = generate_random_cards(3);
                (GameScene::CardSelection(CardSelectionSceneData::new(cards)), SceneTransition::Stay)
            } else {
                (GameScene::DropCollection(data.clone()), SceneTransition::Stay)
            }
        }
        InputEvent::Char(' ') => {
            // Take all items
            while let Some(item) = data.take_selected() {
                apply_loot_item(ctx, &item);
            }
            // Transition to card selection after taking all
            let cards = generate_random_cards(3);
            (GameScene::CardSelection(CardSelectionSceneData::new(cards)), SceneTransition::Stay)
        }
        InputEvent::Cancel => {
            // Skip all items, transition to card selection
            let cards = generate_random_cards(3);
            (GameScene::CardSelection(CardSelectionSceneData::new(cards)), SceneTransition::Stay)
        }
        _ => (GameScene::DropCollection(data.clone()), SceneTransition::Stay)
    }
}

fn handle_card_selection_input(
    data: &mut CardSelectionSceneData,
    ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    match input {
        InputEvent::Up => {
            data.cursor_up();
            (GameScene::CardSelection(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Down => {
            data.cursor_down();
            (GameScene::CardSelection(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Select => {
            // Select card and apply buff
            data.select_current();
            if let Some(card) = data.get_selected_card() {
                ctx.apply_buff_card(card);
            }
            // Proceed to next floor after selecting a card
            proceed_to_next_floor(ctx)
        }
        InputEvent::Cancel => {
            // Skip card selection, proceed to next floor
            proceed_to_next_floor(ctx)
        }
        _ => (GameScene::CardSelection(data.clone()), SceneTransition::Stay)
    }
}

fn handle_floor4_choice_input(
    data: &mut Floor4ChoiceSceneData,
    ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    match input {
        InputEvent::Up => {
            data.cursor_up();
            (GameScene::Floor4Choice(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Down => {
            data.cursor_down();
            (GameScene::Floor4Choice(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Select => {
            // Apply floor 4 choice
            let choice = data.get_selected_choice();
            if let Some(dungeon) = ctx.get_dungeon_mut() {
                dungeon.set_floor4_choice(choice);
                // Get the room and start combat
                if let Some(room) = dungeon.get_current_room() {
                    return (GameScene::Combat(CombatSceneData::from_room(room.clone())), SceneTransition::Stay);
                }
            }
            (GameScene::Floor4Choice(data.clone()), SceneTransition::Stay)
        }
        InputEvent::Cancel => {
            // Go back to title
            (GameScene::Title(TitleSceneData::new()), SceneTransition::Stay)
        }
        _ => (GameScene::Floor4Choice(data.clone()), SceneTransition::Stay)
    }
}

fn handle_result_input(
    _data: ResultSceneData,
    ctx: &mut GameContext,
    input: InputEvent,
) -> (GameScene, SceneTransition) {
    match input {
        InputEvent::Select | InputEvent::Char(' ') => {
            // Return to title
            *ctx = GameContext::new();
            (GameScene::Title(TitleSceneData::new()), SceneTransition::Stay)
        }
        InputEvent::Cancel => {
            (GameScene::Result(_data), SceneTransition::Quit)
        }
        _ => (GameScene::Result(_data), SceneTransition::Stay)
    }
}

/// Apply a loot item to the game context (player gets the item)
fn apply_loot_item(ctx: &mut GameContext, item: &LootItem) {
    match &item.effect {
        ItemEffect::Weapon { attack, ammo } => {
            // Determine weapon effect based on name
            let effect = match item.name.as_str() {
                "Shotgun" => WeaponEffect::Shotgun,
                "Sniper Rifle" => WeaponEffect::Sniper,
                "Electric Gun" => WeaponEffect::Electric,
                _ => WeaponEffect::None,
            };

            // Add weapon to inventory instead of directly equipping
            if let Some(weapon) = item.to_weapon(effect) {
                ctx.add_to_inventory(weapon);
            }
        }
        ItemEffect::Armor(defense) => {
            ctx.player.defense += defense;
        }
        ItemEffect::Consumable(hp) => {
            ctx.player.hp = (ctx.player.hp + hp).min(ctx.player.max_hp);
        }
        ItemEffect::Ammo(amount) => {
            // Refill player's weapon ammo
            if ctx.player.equipped_weapon.max_ammo > 0 {
                ctx.player.equipped_weapon.current_ammo =
                    (ctx.player.equipped_weapon.current_ammo + amount)
                        .min(ctx.player.equipped_weapon.max_ammo + amount);
                ctx.player.equipped_weapon.max_ammo += amount;
            }
        }
    }
}

fn process_combat_turn(ctx: &mut GameContext, data: &mut CombatSceneData) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    data.turn_count += 1;
    data.log(format!("--- Turn {} ---", data.turn_count));

    // Player attacks first enemy
    if ctx.player.is_alive() {
        if let Some(enemy_idx) = data.enemies.iter().position(|e| e.is_alive()) {
            let damage = ctx.player.attack + ctx.player.equipped_weapon.attack;
            let enemy_name = data.enemies[enemy_idx].name.clone();

            data.enemies[enemy_idx].take_damage(damage);
            data.log(format!("{} attacks {} for {} damage!", ctx.player.name, enemy_name, damage));

            if !data.enemies[enemy_idx].is_alive() {
                data.log(format!("{} defeated!", enemy_name));
                ctx.score += 10;
            }
        }
    }

    // Bots attack
    for bot in ctx.bots.iter_mut().filter(|b| b.is_alive()) {
        if let Some(enemy_idx) = data.enemies.iter().position(|e| e.is_alive()) {
            let damage = bot.attack + bot.equipped_weapon.attack;
            let enemy_name = data.enemies[enemy_idx].name.clone();

            data.enemies[enemy_idx].take_damage(damage);
            data.log(format!("{} attacks {} for {} damage!", bot.name, enemy_name, damage));

            if !data.enemies[enemy_idx].is_alive() {
                data.log(format!("{} defeated!", enemy_name));
                ctx.score += 10;
            }
        }
    }

    // Enemies attack (randomly target player or bots)
    let alive_enemies: Vec<(String, i32)> = data.enemies
        .iter()
        .filter(|e| e.is_alive())
        .map(|e| (e.name.clone(), e.attack))
        .collect();

    // Create list of alive party members for targeting
    let mut alive_party: Vec<String> = Vec::new();
    if ctx.player.is_alive() {
        alive_party.push(ctx.player.name.clone());
    }
    for bot in ctx.bots.iter().filter(|b| b.is_alive()) {
        alive_party.push(bot.name.clone());
    }

    for (enemy_name, base_damage) in alive_enemies {
        if alive_party.is_empty() {
            break;
        }

        // Apply room buff damage multiplier
        let damage = (base_damage as f32 * data.room_buff.damage_multiplier()) as i32;

        // Random target
        let target_idx = rng.gen_range(0..alive_party.len());
        let target_name = alive_party[target_idx].clone();

        // Apply damage to target
        if target_name == ctx.player.name {
            ctx.player.take_damage(damage);
            data.log(format!("{} attacks {} for {} damage!", enemy_name, target_name, damage));
            if !ctx.player.is_alive() {
                alive_party.retain(|n| n != &target_name);
            }
        } else {
            // Find the bot
            if let Some(bot) = ctx.bots.iter_mut().find(|b| b.name == target_name) {
                bot.take_damage(damage);
                data.log(format!("{} attacks {} for {} damage!", enemy_name, target_name, damage));
                if !bot.is_alive() {
                    alive_party.retain(|n| n != &target_name);
                }
            }
        }
    }

    // Apply room buff per-turn damage (Contaminated rooms)
    let per_turn_damage = data.room_buff.per_turn_damage();
    if per_turn_damage > 0 {
        if ctx.player.is_alive() {
            ctx.player.take_damage(per_turn_damage);
            data.log(format!("☢️ Contamination damages {} for {} HP!", ctx.player.name, per_turn_damage));
        }
        for bot in ctx.bots.iter_mut().filter(|b| b.is_alive()) {
            bot.take_damage(per_turn_damage);
            data.log(format!("☢️ Contamination damages {} for {} HP!", bot.name, per_turn_damage));
        }
    }

    // Check game over conditions
    if !ctx.is_party_alive() {
        data.log("Your party has been defeated...".to_string());
    }
}

/// Equip weapon to target (player or bot)
fn equip_weapon_to_target(
    ctx: &mut GameContext,
    target: &EquipTarget,
    weapon: super::entities::Weapon,
    data: &mut CombatSceneData,
) {
    match target {
        EquipTarget::Player => {
            ctx.player.equipped_weapon = weapon.clone();
            data.log(format!("Player equipped {}!", weapon.name));
        }
        EquipTarget::Bot(idx) => {
            if let Some(bot) = ctx.bots.get_mut(*idx) {
                if bot.is_alive() {
                    bot.equipped_weapon = weapon.clone();
                    data.log(format!("{} equipped {}!", bot.name, weapon.name));
                }
            }
        }
    }
}

/// Proceed to next floor in the dungeon
fn proceed_to_next_floor(ctx: &mut GameContext) -> (GameScene, SceneTransition) {
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
fn generate_drops(enemies: &[super::entities::Enemy], loot_multiplier: f32) -> Vec<LootItem> {
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
