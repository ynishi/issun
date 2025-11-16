//! Combat scene data

use crate::models::entities::{Enemy, RoomBuff, Weapon};
use crate::models::{GameContext, GameScene, proceed_to_next_floor, scene_helpers::generate_drops, scenes::{DropCollectionSceneData, ResultSceneData}};
use issun::prelude::SceneTransition;
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSceneData {
    pub enemies: Vec<Enemy>,
    pub combat_log: Vec<String>,
    pub turn_count: u32,
    /// Show inventory for weapon switching
    pub show_inventory: bool,
    /// Selected inventory index
    pub inventory_cursor: usize,
    /// Target for equipment (Player or Bot index)
    pub equip_target: EquipTarget,
    /// Room buff affecting this combat
    pub room_buff: RoomBuff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EquipTarget {
    Player,
    Bot(usize),
}

impl CombatSceneData {
    pub fn new(enemies: Vec<Enemy>) -> Self {
        Self {
            enemies,
            combat_log: Vec::new(),
            turn_count: 0,
            show_inventory: false,
            inventory_cursor: 0,
            equip_target: EquipTarget::Player,
            room_buff: RoomBuff::Normal,
        }
    }

    /// Create combat scene from a room
    pub fn from_room(room: crate::models::entities::Room) -> Self {
        Self {
            enemies: room.enemies,
            combat_log: Vec::new(),
            turn_count: 0,
            show_inventory: false,
            inventory_cursor: 0,
            equip_target: EquipTarget::Player,
            room_buff: room.buff,
        }
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.combat_log.push(message.into());
    }

    /// Toggle inventory display
    pub fn toggle_inventory(&mut self) {
        self.show_inventory = !self.show_inventory;
        if self.show_inventory {
            self.inventory_cursor = 0;
        }
    }

    /// Move inventory cursor up
    pub fn move_inventory_up(&mut self) {
        if self.inventory_cursor > 0 {
            self.inventory_cursor -= 1;
        }
    }

    /// Move inventory cursor down
    pub fn move_inventory_down(&mut self, max: usize) {
        if self.inventory_cursor < max.saturating_sub(1) {
            self.inventory_cursor += 1;
        }
    }

    /// Cycle equip target (Player -> Bot0 -> Bot1 -> Player)
    pub fn cycle_equip_target(&mut self, bot_count: usize) {
        self.equip_target = match self.equip_target {
            EquipTarget::Player => {
                if bot_count > 0 {
                    EquipTarget::Bot(0)
                } else {
                    EquipTarget::Player
                }
            }
            EquipTarget::Bot(idx) => {
                if idx + 1 < bot_count {
                    EquipTarget::Bot(idx + 1)
                } else {
                    EquipTarget::Player
                }
            }
        };
    }

    pub fn handle_input(
        mut self,
        ctx: &mut GameContext,
        input: InputEvent,
    ) -> (GameScene, SceneTransition) {
        match input {
            InputEvent::Cancel => {
                (GameScene::Combat(self), SceneTransition::Quit)
            }
            InputEvent::Char('i') | InputEvent::Char('I') => {
                // Toggle inventory
                self.toggle_inventory();
                (GameScene::Combat(self), SceneTransition::Stay)
            }
            InputEvent::Tab => {
                // Cycle equip target when inventory is shown
                if self.show_inventory {
                    let bot_count = ctx.bots.iter().filter(|b| b.is_alive()).count();
                    self.cycle_equip_target(bot_count);
                }
                (GameScene::Combat(self), SceneTransition::Stay)
            }
            InputEvent::Up => {
                if self.show_inventory {
                    self.move_inventory_up();
                }
                (GameScene::Combat(self), SceneTransition::Stay)
            }
            InputEvent::Down => {
                if self.show_inventory {
                    self.move_inventory_down(ctx.inventory.len());
                }
                (GameScene::Combat(self), SceneTransition::Stay)
            }
            InputEvent::Select => {
                // Equip weapon when inventory is shown
                if self.show_inventory && !ctx.inventory.is_empty() {
                    if self.inventory_cursor < ctx.inventory.len() {
                        let weapon = ctx.inventory[self.inventory_cursor].clone();
                        let target = self.equip_target.clone();
                        self.equip_weapon_to_target(ctx, &target, weapon);
                    }
                    self.show_inventory = false;
                }
                (GameScene::Combat(self), SceneTransition::Stay)
            }
            InputEvent::Char(' ') => {
                // Process combat turn
                self.process_turn(ctx);

                // Check win/lose conditions
                let all_enemies_dead = self.enemies.iter().all(|e| !e.is_alive());
                let party_dead = !ctx.is_party_alive();

                if all_enemies_dead {
                    // Floor completed! Generate drops with room buff multiplier
                    let drops = generate_drops(&self.enemies, self.room_buff.loot_multiplier());

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
                    (GameScene::Combat(self), SceneTransition::Stay)
                }
            }
            _ => (GameScene::Combat(self), SceneTransition::Stay)
        }
    }

    /// Process a combat turn
    fn process_turn(&mut self, ctx: &mut GameContext) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        self.turn_count += 1;
        self.log(format!("--- Turn {} ---", self.turn_count));

        // Player attacks first enemy
        if ctx.player.is_alive() {
            if let Some(enemy_idx) = self.enemies.iter().position(|e| e.is_alive()) {
                let damage = ctx.player.attack + ctx.player.equipped_weapon.attack;
                let enemy_name = self.enemies[enemy_idx].name.clone();

                self.enemies[enemy_idx].take_damage(damage);
                self.log(format!("{} attacks {} for {} damage!", ctx.player.name, enemy_name, damage));

                if !self.enemies[enemy_idx].is_alive() {
                    self.log(format!("{} defeated!", enemy_name));
                    ctx.score += 10;
                }
            }
        }

        // Bots attack
        for bot in ctx.bots.iter_mut().filter(|b| b.is_alive()) {
            if let Some(enemy_idx) = self.enemies.iter().position(|e| e.is_alive()) {
                let damage = bot.attack + bot.equipped_weapon.attack;
                let enemy_name = self.enemies[enemy_idx].name.clone();

                self.enemies[enemy_idx].take_damage(damage);
                self.log(format!("{} attacks {} for {} damage!", bot.name, enemy_name, damage));

                if !self.enemies[enemy_idx].is_alive() {
                    self.log(format!("{} defeated!", enemy_name));
                    ctx.score += 10;
                }
            }
        }

        // Enemies attack (randomly target player or bots)
        let alive_enemies: Vec<(String, i32)> = self.enemies
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
            let damage = (base_damage as f32 * self.room_buff.damage_multiplier()) as i32;

            // Random target
            let target_idx = rng.gen_range(0..alive_party.len());
            let target_name = alive_party[target_idx].clone();

            // Apply damage to target
            if target_name == ctx.player.name {
                ctx.player.take_damage(damage);
                self.log(format!("{} attacks {} for {} damage!", enemy_name, target_name, damage));
                if !ctx.player.is_alive() {
                    alive_party.retain(|n| n != &target_name);
                }
            } else {
                // Find the bot
                if let Some(bot) = ctx.bots.iter_mut().find(|b| b.name == target_name) {
                    bot.take_damage(damage);
                    self.log(format!("{} attacks {} for {} damage!", enemy_name, target_name, damage));
                    if !bot.is_alive() {
                        alive_party.retain(|n| n != &target_name);
                    }
                }
            }
        }

        // Apply room buff per-turn damage (Contaminated rooms)
        let per_turn_damage = self.room_buff.per_turn_damage();
        if per_turn_damage > 0 {
            if ctx.player.is_alive() {
                ctx.player.take_damage(per_turn_damage);
                self.log(format!("☢️ Contamination damages {} for {} HP!", ctx.player.name, per_turn_damage));
            }
            for bot in ctx.bots.iter_mut().filter(|b| b.is_alive()) {
                bot.take_damage(per_turn_damage);
                self.log(format!("☢️ Contamination damages {} for {} HP!", bot.name, per_turn_damage));
            }
        }

        // Check game over conditions
        if !ctx.is_party_alive() {
            self.log("Your party has been defeated...".to_string());
        }
    }

    /// Equip weapon to target (player or bot)
    fn equip_weapon_to_target(
        &mut self,
        ctx: &mut GameContext,
        target: &EquipTarget,
        weapon: Weapon,
    ) {
        match target {
            EquipTarget::Player => {
                ctx.player.equipped_weapon = weapon.clone();
                self.log(format!("Player equipped {}!", weapon.name));
            }
            EquipTarget::Bot(idx) => {
                if let Some(bot) = ctx.bots.get_mut(*idx) {
                    if bot.is_alive() {
                        bot.equipped_weapon = weapon.clone();
                        self.log(format!("{} equipped {}!", bot.name, weapon.name));
                    }
                }
            }
        }
    }
}
