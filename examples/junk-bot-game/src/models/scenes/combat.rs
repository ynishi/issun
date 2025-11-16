//! Combat scene data

use crate::models::entities::{Enemy, RoomBuff};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSceneData {
    pub enemies: Vec<Enemy>,
    pub combat_log: Vec<String>,
    pub turn_count: u32,
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
            combat_log: Vec::new(),
            turn_count: 0,
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
            combat_log: Vec::new(),
            turn_count: 0,
            show_inventory: false,
            inventory_cursor: 0,
            equip_target: EquipTarget::Player,
            room_buff: room.buff,
        }
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.combat_log.push(message.into());
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
}
