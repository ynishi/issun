//! Game content data layer
//!
//! Static/const data or data loaded from files

use issun::prelude::*;
use issun::Asset; // Import derive macro

/// Enemy asset definition
#[derive(Debug, Clone, Asset)]
pub struct EnemyAsset {
    pub name: &'static str,
    pub hp: i32,
    pub attack: i32,
}

/// Example enemy preset data
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
