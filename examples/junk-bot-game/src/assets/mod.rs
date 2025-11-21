//! Game content data layer
//!
//! Static/const data or data loaded from files

use issun::prelude::*;
use issun::{Asset, Resource}; // Import derive macros

use crate::models::entities::{BuffType, Rarity};

/// Enemy asset definition
#[derive(Debug, Clone, Asset)]
#[allow(dead_code)]
pub struct EnemyAsset {
    pub name: &'static str,
    pub hp: i32,
    pub attack: i32,
}

/// Example enemy preset data
#[allow(dead_code)]
pub const ENEMIES: &[EnemyAsset] = &[
    EnemyAsset {
        name: "Goblin",
        hp: 30,
        attack: 5,
    },
    EnemyAsset {
        name: "Orc",
        hp: 50,
        attack: 10,
    },
    EnemyAsset {
        name: "Dragon",
        hp: 100,
        attack: 20,
    },
];

/// Buff card asset definition
#[derive(Debug, Clone)]
pub struct BuffCardAsset {
    pub name: &'static str,
    pub description: &'static str,
    pub buff_type: BuffType,
    pub rarity: Rarity,
}

/// Buff card database (for Resources)
#[derive(Debug, Clone, Resource)]
#[allow(dead_code)]
pub struct BuffCardDatabase {
    pub cards: Vec<BuffCardAsset>,
}

impl BuffCardDatabase {
    /// Create database from const data
    pub fn new() -> Self {
        Self {
            cards: BUFF_CARDS.to_vec(),
        }
    }
}

impl Default for BuffCardDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// All available buff cards (const data)
pub const BUFF_CARDS: &[BuffCardAsset] = &[
    BuffCardAsset {
        name: "Rusty Blade",
        description: "Attack +5",
        buff_type: BuffType::AttackUp(5),
        rarity: Rarity::Common,
    },
    BuffCardAsset {
        name: "Junk Armor",
        description: "Max HP +30",
        buff_type: BuffType::HpUp(30),
        rarity: Rarity::Common,
    },
    BuffCardAsset {
        name: "Sharp Edge",
        description: "Attack +10",
        buff_type: BuffType::AttackUp(10),
        rarity: Rarity::Uncommon,
    },
    BuffCardAsset {
        name: "Reinforced Plating",
        description: "Max HP +50",
        buff_type: BuffType::HpUp(50),
        rarity: Rarity::Uncommon,
    },
    BuffCardAsset {
        name: "Looter's Dream",
        description: "Drop Rate +100%",
        buff_type: BuffType::DropRateUp(1.0),
        rarity: Rarity::Rare,
    },
    BuffCardAsset {
        name: "Critical Strike",
        description: "Critical Rate +20%",
        buff_type: BuffType::CriticalUp(0.2),
        rarity: Rarity::Rare,
    },
    BuffCardAsset {
        name: "Heavy Hitter",
        description: "Attack +15",
        buff_type: BuffType::AttackUp(15),
        rarity: Rarity::Rare,
    },
    BuffCardAsset {
        name: "Turbo Boost",
        description: "Speed +10",
        buff_type: BuffType::SpeedUp(10),
        rarity: Rarity::Epic,
    },
    BuffCardAsset {
        name: "Titan's Strength",
        description: "Attack +20",
        buff_type: BuffType::AttackUp(20),
        rarity: Rarity::Epic,
    },
    BuffCardAsset {
        name: "Legendary Power",
        description: "Attack +30",
        buff_type: BuffType::AttackUp(30),
        rarity: Rarity::Legendary,
    },
];
