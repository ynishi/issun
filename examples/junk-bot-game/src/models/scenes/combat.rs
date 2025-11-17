//! Combat scene data

use crate::models::entities::{Enemy, RoomBuff, Weapon};
use crate::models::{
    proceed_to_next_floor,
    scene_helpers::generate_drops,
    scenes::{DropCollectionSceneData, ResultSceneData},
    GameContext,
    GameScene,
};
use issun::prelude::{
    CombatService,
    CombatSystem,
    Combatant,
    ResourceContext,
    SceneTransition,
    ServiceContext,
    SystemContext,
    TurnBasedCombatConfig,
};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSceneData {
    pub enemies: Vec<Enemy>,
    /// Combat engine managing turn count, log, and score
    #[serde(skip)]
    pub combat_engine: Option<CombatSystem>,
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
            combat_engine: Some(CombatSystem::new(TurnBasedCombatConfig::default())),
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
            combat_engine: Some(CombatSystem::new(TurnBasedCombatConfig::default())),
            show_inventory: false,
            inventory_cursor: 0,
            equip_target: EquipTarget::Player,
            room_buff: room.buff,
        }
    }

    /// Get combat engine (lazily initialized)
    fn engine(&mut self) -> &mut CombatSystem {
        self.combat_engine.get_or_insert_with(|| {
            CombatSystem::new(TurnBasedCombatConfig::default())
        })
    }

    /// Get combat log
    pub fn combat_log(&self) -> &[issun::prelude::CombatLogEntry] {
        self.combat_engine.as_ref()
            .map(|e| e.log())
            .unwrap_or(&[])
    }

    /// Get turn count
    pub fn turn_count(&self) -> u32 {
        self.combat_engine.as_ref()
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
            InputEvent::Cancel => {
                SceneTransition::Quit
            }
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
                        self.equip_weapon_to_target(ctx, &target, weapon);
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
                    return SceneTransition::Switch(GameScene::DropCollection(
                        DropCollectionSceneData::new(drops),
                    ));
                } else if party_dead {
                    SceneTransition::Switch(GameScene::Result(ResultSceneData::new(false, ctx.score)))
                } else {
                    SceneTransition::Stay
                }
            }
            _ => SceneTransition::Stay
        }
    }

    /// Process a combat turn using CombatSystem
    fn process_turn(&mut self, ctx: &mut GameContext, services: &ServiceContext) {
        if let Some(combat_service) = services.get_as::<CombatService>("combat_service") {
            let demo_damage = combat_service.calculate_damage(100, Some(20));
            self.log(format!(
                "🔧 Service Registry Demo: 100 dmg - 20 def = {}",
                demo_damage
            ));
        }

        // Build party trait object slice (player + bots)
        let mut party: Vec<&mut dyn Combatant> = vec![&mut ctx.player as &mut dyn Combatant];
        for bot in ctx.bots.iter_mut() {
            party.push(bot as &mut dyn Combatant);
        }

        // Build enemy trait object slice
        let mut enemies: Vec<&mut dyn Combatant> = self.enemies
            .iter_mut()
            .map(|e| e as &mut dyn Combatant)
            .collect();

        // Process turn using combat engine (CombatSystem internally uses CombatService)
        let damage_multiplier = self.room_buff.damage_multiplier();
        let per_turn_damage = self.room_buff.per_turn_damage();

        // Extract engine to avoid double borrow
        let engine = self.combat_engine.as_mut().unwrap();

        let _result = engine.process_turn_dyn(
            &mut party,
            &mut enemies,
            damage_multiplier,
            per_turn_damage,
        );

        // Sync score from engine to context
        ctx.score = engine.score();
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
