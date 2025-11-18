//! Player entity (minimal example)

use issun::prelude::*;
use issun::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
pub struct Player {
    #[entity(id)]
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hp: 100,
            max_hp: 150,
        }
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}
