//! Player entity

use super::weapon::Weapon;
use issun::prelude::*;
use issun::Entity; // Import derive macro
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
pub struct Player {
    #[entity(id)]
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub equipped_weapon: Weapon,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hp: 100,
            max_hp: 100,
            attack: 10,
            defense: 5,
            equipped_weapon: Weapon::default_weapon(),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    #[allow(dead_code)]
    pub fn take_damage(&mut self, damage: i32) {
        let actual_damage = (damage - self.defense).max(1);
        self.hp = (self.hp - actual_damage).max(0);
    }
}

impl Combatant for Player {
    fn name(&self) -> &str {
        &self.name
    }

    fn hp(&self) -> i32 {
        self.hp
    }

    fn max_hp(&self) -> i32 {
        self.max_hp
    }

    fn attack(&self) -> i32 {
        self.attack + self.equipped_weapon.attack
    }

    fn defense(&self) -> Option<i32> {
        Some(self.defense)
    }

    fn take_damage(&mut self, damage: i32) {
        // Now just applies raw damage (CombatService handles defense calculation)
        self.hp = (self.hp - damage).max(0);
    }
}
