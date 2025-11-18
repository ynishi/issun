//! Game context - persistent data across scenes
//!
//! This data survives scene transitions and should be saved/loaded

use super::entities::{
    Bot, BuffCard, BuffType, Dungeon, ItemEffect, LootItem, Player, Weapon, WeaponEffect,
};
use issun::prelude::*;
use serde::{Deserialize, Serialize};

/// Persistent game data (survives scene transitions)
#[derive(Serialize, Deserialize)]
pub struct GameContext {
    pub player: Player,
    pub bots: Vec<Bot>,
    pub inventory: Vec<Weapon>,
    pub buff_cards: Vec<BuffCard>,
    pub score: u32,
    pub floor: u32,
    pub dungeon: Option<Dungeon>,
}

impl Clone for GameContext {
    fn clone(&self) -> Self {
        Self {
            player: self.player.clone(),
            bots: self.bots.clone(),
            inventory: self.inventory.clone(),
            buff_cards: self.buff_cards.clone(),
            score: self.score,
            floor: self.floor,
            dungeon: self.dungeon.clone(),
        }
    }
}

impl std::fmt::Debug for GameContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameContext")
            .field("player", &self.player)
            .field("bots", &self.bots)
            .field("inventory", &self.inventory)
            .field("buff_cards", &self.buff_cards)
            .field("score", &self.score)
            .field("floor", &self.floor)
            .field("dungeon", &self.dungeon)
            .finish()
    }
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            player: Player::new("Hero"),
            bots: vec![Bot::new("Rusty"), Bot::new("Sparky")],
            inventory: Vec::new(),
            buff_cards: Vec::new(),
            score: 0,
            floor: 1,
            dungeon: None,
        }
    }

    /// Start a new dungeon run
    pub fn start_dungeon(&mut self) {
        self.dungeon = Some(Dungeon::new());
        self.floor = 1;
    }

    /// Get current dungeon
    pub fn get_dungeon(&self) -> Option<&Dungeon> {
        self.dungeon.as_ref()
    }

    /// Get mutable dungeon
    pub fn get_dungeon_mut(&mut self) -> Option<&mut Dungeon> {
        self.dungeon.as_mut()
    }

    /// Check if at least one party member (player or bot) is alive
    pub fn is_party_alive(&self) -> bool {
        self.player.is_alive() || self.bots.iter().any(|bot| bot.is_alive())
    }

    /// Get all alive bots
    pub fn alive_bots(&self) -> Vec<&Bot> {
        self.bots.iter().filter(|bot| bot.is_alive()).collect()
    }

    /// Get mutable reference to alive bots
    pub fn alive_bots_mut(&mut self) -> Vec<&mut Bot> {
        self.bots.iter_mut().filter(|bot| bot.is_alive()).collect()
    }

    /// Add weapon to inventory
    pub fn add_to_inventory(&mut self, weapon: Weapon) {
        self.inventory.push(weapon);
    }

    /// Remove weapon from inventory by index
    pub fn remove_from_inventory(&mut self, index: usize) -> Option<Weapon> {
        if index < self.inventory.len() {
            Some(self.inventory.remove(index))
        } else {
            None
        }
    }

    /// Apply a buff card to the player and bots
    pub fn apply_buff_card(&mut self, card: BuffCard) {
        match &card.buff_type {
            BuffType::AttackUp(amount) => {
                self.player.attack += amount;
                for bot in &mut self.bots {
                    bot.attack += amount;
                }
            }
            BuffType::HpUp(amount) => {
                self.player.max_hp += amount;
                self.player.hp += amount; // Also heal by the amount
                for bot in &mut self.bots {
                    bot.max_hp += amount;
                    bot.hp += amount;
                }
            }
            BuffType::DropRateUp(_multiplier) => {
                // TODO: Implement drop rate system in the future
                // For now, just store the card
            }
            BuffType::CriticalUp(_rate) => {
                // TODO: Implement critical hit system in the future
                // For now, just store the card
            }
            BuffType::SpeedUp(_amount) => {
                // TODO: Implement speed system in the future
                // For now, just store the card
            }
        }
        self.buff_cards.push(card);
    }

    /// Apply a loot item to the player
    pub fn apply_loot_item(&mut self, item: &LootItem) {
        match &item.effect {
            ItemEffect::Weapon { attack: _, ammo: _ } => {
                // Determine weapon effect based on name
                let effect = match item.name.as_str() {
                    "Shotgun" => WeaponEffect::Shotgun,
                    "Sniper Rifle" => WeaponEffect::Sniper,
                    "Electric Gun" => WeaponEffect::Electric,
                    _ => WeaponEffect::None,
                };

                // Add weapon to inventory instead of directly equipping
                if let Some(weapon) = item.to_weapon(effect) {
                    self.add_to_inventory(weapon);
                }
            }
            ItemEffect::Armor(defense) => {
                self.player.defense += defense;
            }
            ItemEffect::Consumable(hp) => {
                self.player.hp = (self.player.hp + hp).min(self.player.max_hp);
            }
            ItemEffect::Ammo(amount) => {
                // Refill player's weapon ammo
                if self.player.equipped_weapon.max_ammo > 0 {
                    self.player.equipped_weapon.current_ammo =
                        (self.player.equipped_weapon.current_ammo + amount)
                            .min(self.player.equipped_weapon.max_ammo + amount);
                    self.player.equipped_weapon.max_ammo += amount;
                }
            }
        }
    }
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}
