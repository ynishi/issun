//! Combat scene data

use crate::models::entities::Enemy;

#[derive(Debug)]
pub struct CombatSceneData {
    pub enemies: Vec<Enemy>,
    pub combat_log: Vec<String>,
    pub turn_count: u32,
}

impl CombatSceneData {
    pub fn new(enemies: Vec<Enemy>) -> Self {
        Self {
            enemies,
            combat_log: Vec::new(),
            turn_count: 0,
        }
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.combat_log.push(message.into());
    }
}
