//! Bot entity - Companion characters that fight alongside the player

use super::weapon::Weapon;
use issun::prelude::*;
use issun::Entity; // Import derive macro
use serde::{Deserialize, Serialize};

/// Bot state - tracks what the bot is currently doing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BotState {
    Idle,
    Combat,
    Following,
}

/// Bot companion entity
///
/// Bots are AI companions that fight alongside the player.
/// They have their own stats, weapons, and can be upgraded.
#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
pub struct Bot {
    /// Unique identifier (bot name)
    #[entity(id)]
    pub name: String,

    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub state: BotState,
    pub equipped_weapon: Weapon,
}

impl Bot {
    /// Create a new bot with default stats
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hp: 50,
            max_hp: 50,
            attack: 8,
            state: BotState::Idle,
            equipped_weapon: Weapon::default_weapon(),
        }
    }

    /// Create a bot with custom stats
    pub fn with_stats(name: impl Into<String>, hp: i32, attack: i32) -> Self {
        Self {
            name: name.into(),
            hp,
            max_hp: hp,
            attack,
            state: BotState::Idle,
            equipped_weapon: Weapon::default_weapon(),
        }
    }

    /// Check if bot is still alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Take damage (no defense for bots)
    pub fn take_damage(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(0);
    }

    /// Heal the bot
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Set bot state
    pub fn set_state(&mut self, state: BotState) {
        self.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_creation() {
        let bot = Bot::new("Rusty");
        assert_eq!(bot.name, "Rusty");
        assert_eq!(bot.hp, 50);
        assert_eq!(bot.max_hp, 50);
        assert_eq!(bot.attack, 8);
        assert!(bot.is_alive());
    }

    #[test]
    fn test_bot_take_damage() {
        let mut bot = Bot::new("Rusty");
        bot.take_damage(20);
        assert_eq!(bot.hp, 30);
        assert!(bot.is_alive());

        bot.take_damage(100);
        assert_eq!(bot.hp, 0);
        assert!(!bot.is_alive());
    }

    #[test]
    fn test_bot_heal() {
        let mut bot = Bot::new("Rusty");
        bot.take_damage(30);
        assert_eq!(bot.hp, 20);

        bot.heal(15);
        assert_eq!(bot.hp, 35);

        bot.heal(100);
        assert_eq!(bot.hp, 50); // Capped at max_hp
    }
}
