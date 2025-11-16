//! Enemy entity

use issun::prelude::*;
use issun::Entity; // Import derive macro

#[derive(Debug, Clone, Entity)]
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
