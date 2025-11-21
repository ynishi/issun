//! Combat scene data

use crate::models::entities::{Enemy, RoomBuff, Weapon};
use crate::models::{
    proceed_to_next_floor,
    scene_helpers::generate_drops,
    scenes::{DropCollectionSceneData, ResultSceneData},
    GameContext, GameScene,
};
use issun::prelude::{
    CombatService, Combatant, ResourceContext, SceneTransition, ServiceContext, SystemContext,
};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

/// Simple combat engine for junk-bot-game
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SimpleCombatEngine {
    turn_count: u32,
    log: Vec<String>,
    score: u32,
}

impl SimpleCombatEngine {
    fn new() -> Self {
        Self::default()
    }

    fn add_log(&mut self, message: String) {
        self.log.push(message);
    }

    fn log(&self) -> &[String] {
        &self.log
    }

    fn turn_count(&self) -> u32 {
        self.turn_count
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn add_score(&mut self, points: u32) {
        self.score += points;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSceneData {
    pub enemies: Vec<Enemy>,
    /// Combat engine managing turn count, log, and score
    #[serde(skip)]
    pub combat_engine: Option<SimpleCombatEngine>,
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
            combat_engine: Some(SimpleCombatEngine::new()),
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
            combat_engine: Some(SimpleCombatEngine::new()),
            show_inventory: false,
            inventory_cursor: 0,
            equip_target: EquipTarget::Player,
            room_buff: room.buff,
        }
    }

    /// Get combat engine (lazily initialized)
    fn engine(&mut self) -> &mut SimpleCombatEngine {
        self.combat_engine
            .get_or_insert_with(|| SimpleCombatEngine::new())
    }

    /// Get combat log
    pub fn combat_log(&self) -> &[String] {
        self.combat_engine.as_ref().map(|e| e.log()).unwrap_or(&[])
    }

    /// Get turn count
    pub fn turn_count(&self) -> u32 {
        self.combat_engine
            .as_ref()
            .map(|e| e.turn_count())
            .unwrap_or(0)
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.engine().add_log(message.into());
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

    pub async fn handle_input(
        &mut self,
        services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        let mut ctx = resources
            .get_mut::<GameContext>()
            .await
            .expect("GameContext resource not registered");
        match input {
            InputEvent::Cancel => SceneTransition::Quit,
            InputEvent::Char('i') | InputEvent::Char('I') => {
                // Toggle inventory
                self.toggle_inventory();
                SceneTransition::Stay
            }
            InputEvent::Tab => {
                // Cycle equip target when inventory is shown
                if self.show_inventory {
                    let bot_count = ctx.bots.iter().filter(|b| b.is_alive()).count();
                    self.cycle_equip_target(bot_count);
                }
                SceneTransition::Stay
            }
            InputEvent::Up => {
                if self.show_inventory {
                    self.move_inventory_up();
                }
                SceneTransition::Stay
            }
            InputEvent::Down => {
                if self.show_inventory {
                    self.move_inventory_down(ctx.inventory.len());
                }
                SceneTransition::Stay
            }
            InputEvent::Select => {
                // Equip weapon when inventory is shown
                if self.show_inventory && !ctx.inventory.is_empty() {
                    if self.inventory_cursor < ctx.inventory.len() {
                        let weapon = ctx.inventory[self.inventory_cursor].clone();
                        let target = self.equip_target.clone();
                        self.equip_weapon_to_target(&mut ctx, &target, weapon);
                    }
                    self.show_inventory = false;
                }
                SceneTransition::Stay
            }
            InputEvent::Char(' ') => {
                // Process combat turn
                self.process_turn(&mut ctx, services);

                // Check win/lose conditions
                let all_enemies_dead = self.enemies.iter().all(|e| !e.is_alive());
                let party_dead = !ctx.is_party_alive();

                if all_enemies_dead {
                    // Floor completed! Generate drops with room buff multiplier
                    let drops = generate_drops(&self.enemies, self.room_buff.loot_multiplier());

                    if drops.is_empty() {
                        drop(ctx);
                        return proceed_to_next_floor(resources).await;
                    }
                    // Show drop collection scene
                    SceneTransition::Switch(GameScene::DropCollection(
                        DropCollectionSceneData::new(drops),
                    ))
                } else if party_dead {
                    SceneTransition::Switch(GameScene::Result(ResultSceneData::new(
                        false, ctx.score,
                    )))
                } else {
                    SceneTransition::Stay
                }
            }
            _ => SceneTransition::Stay,
        }
    }

    /// Process a combat turn using simplified combat logic
    fn process_turn(&mut self, ctx: &mut GameContext, services: &ServiceContext) {
        let engine = self.engine();
        engine.turn_count += 1;

        // Get combat service for damage calculations
        let combat_service = services.get_as::<CombatService>("combat_service");

        // Apply room buff effects
        let damage_multiplier = self.room_buff.damage_multiplier();
        let per_turn_damage = self.room_buff.per_turn_damage();

        // Collect log messages to avoid borrow checker issues
        let mut log_messages = Vec::new();

        // Party attacks enemies (player first)
        let mut score_to_add = 0;
        if ctx.player.is_alive() {
            if let Some(target) = self.enemies.iter_mut().find(|e| e.is_alive()) {
                let base_damage = ctx.player.equipped_weapon.attack;
                let damage = if let Some(service) = combat_service {
                    service.calculate_damage(
                        (base_damage as f32 * damage_multiplier) as i32,
                        target.defense(),
                    )
                } else {
                    ((base_damage as f32 * damage_multiplier) as i32).saturating_sub(target.defense().unwrap_or(0))
                };

                target.hp = target.hp.saturating_sub(damage);
                log_messages.push(format!(
                    "{} attacks {} for {} damage!",
                    ctx.player.name,
                    target.name,
                    damage
                ));

                if target.hp == 0 {
                    log_messages.push(format!("{} defeated!", target.name));
                    score_to_add += 100;
                }
            }
        }

        // Then bots attack
        for bot in ctx.bots.iter_mut() {
            if bot.is_alive() {
                if let Some(target) = self.enemies.iter_mut().find(|e| e.is_alive()) {
                    let base_damage = bot.equipped_weapon.attack;
                    let damage = if let Some(service) = combat_service {
                        service.calculate_damage(
                            (base_damage as f32 * damage_multiplier) as i32,
                            target.defense(),
                        )
                    } else {
                        ((base_damage as f32 * damage_multiplier) as i32).saturating_sub(target.defense().unwrap_or(0))
                    };

                    target.hp = target.hp.saturating_sub(damage);
                    log_messages.push(format!(
                        "{} attacks {} for {} damage!",
                        bot.name,
                        target.name,
                        damage
                    ));

                    if target.hp == 0 {
                        log_messages.push(format!("{} defeated!", target.name));
                        score_to_add += 100;
                    }
                }
            }
        }

        // Enemies attack party
        for enemy in self.enemies.iter_mut() {
            if enemy.is_alive() {
                // Find first alive party member
                if ctx.player.is_alive() {
                    let damage = if let Some(service) = combat_service {
                        service.calculate_damage(enemy.attack, ctx.player.defense())
                    } else {
                        enemy.attack.saturating_sub(ctx.player.defense().unwrap_or(0))
                    };

                    ctx.player.hp = ctx.player.hp.saturating_sub(damage);
                    log_messages.push(format!(
                        "{} attacks {} for {} damage!",
                        enemy.name,
                        ctx.player.name,
                        damage
                    ));

                    if ctx.player.hp == 0 {
                        log_messages.push(format!("{} defeated!", ctx.player.name));
                    }
                } else if let Some(bot) = ctx.bots.iter_mut().find(|b| b.is_alive()) {
                    let damage = if let Some(service) = combat_service {
                        service.calculate_damage(enemy.attack, bot.defense())
                    } else {
                        enemy.attack.saturating_sub(bot.defense().unwrap_or(0))
                    };

                    bot.hp = bot.hp.saturating_sub(damage);
                    log_messages.push(format!(
                        "{} attacks {} for {} damage!",
                        enemy.name,
                        bot.name,
                        damage
                    ));

                    if bot.hp == 0 {
                        log_messages.push(format!("{} defeated!", bot.name));
                    }
                }
            }
        }

        // Apply per-turn damage from room buff
        if per_turn_damage > 0 {
            if ctx.player.is_alive() {
                ctx.player.hp = ctx.player.hp.saturating_sub(per_turn_damage);
                log_messages.push(format!(
                    "Room effect deals {} damage to {}!",
                    per_turn_damage,
                    ctx.player.name
                ));
            }
            for bot in ctx.bots.iter_mut() {
                if bot.is_alive() {
                    bot.hp = bot.hp.saturating_sub(per_turn_damage);
                    log_messages.push(format!(
                        "Room effect deals {} damage to {}!",
                        per_turn_damage,
                        bot.name
                    ));
                }
            }
        }

        // Add all log messages and score at once
        for msg in log_messages {
            self.engine().add_log(msg);
        }
        self.engine().add_score(score_to_add);

        // Sync score
        ctx.score = self.engine().score();
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
