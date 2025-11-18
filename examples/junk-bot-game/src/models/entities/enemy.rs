//! Enemy entity

use issun::prelude::*;
use issun::Entity; // Import derive macro
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
pub struct Enemy {
    #[entity(id)]
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
}

impl Enemy {
    pub fn new(name: impl Into<String>, hp: i32, attack: i32) -> Self {
        Self {
            name: name.into(),
            hp,
            max_hp: hp,
            attack,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(0);
    }
}

impl Combatant for Enemy {
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
        self.attack
    }

    fn defense(&self) -> Option<i32> {
        None // Enemies have no defense (can be added per enemy type later)
    }

    fn take_damage(&mut self, damage: i32) {
        // Now just applies raw damage (CombatService handles defense calculation)
        self.hp = (self.hp - damage).max(0);
    }
}
