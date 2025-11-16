//! Player entity

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
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hp: 100,
            max_hp: 100,
            attack: 10,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(0);
    }
}
