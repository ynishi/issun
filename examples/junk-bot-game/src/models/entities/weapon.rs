//! Weapon entity

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeaponEffect {
    None,
    Shotgun,  // Hits multiple enemies but lower damage
    Sniper,   // High damage but skips next turn
    Electric, // Low damage but can stun enemies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub attack: i32,
    pub max_ammo: i32,
    pub current_ammo: i32,
    pub effect: WeaponEffect,
}

impl Weapon {
    /// Create a new weapon
    pub fn new(name: impl Into<String>, attack: i32, max_ammo: i32) -> Self {
        Self {
            name: name.into(),
            attack,
            max_ammo,
            current_ammo: max_ammo,
            effect: WeaponEffect::None,
        }
    }

    /// Create default weapon (infinite ammo)
    pub fn default_weapon() -> Self {
        Self {
            name: "Rusty Pistol".into(),
            attack: 5,
            max_ammo: -1, // Infinite ammo
            current_ammo: -1,
            effect: WeaponEffect::None,
        }
    }

    /// Check if weapon has ammo (infinite weapons always return true)
    pub fn has_ammo(&self) -> bool {
        self.max_ammo < 0 || self.current_ammo > 0
    }

    /// Use ammo (does nothing for infinite weapons)
    pub fn use_ammo(&mut self) {
        if self.max_ammo >= 0 {
            self.current_ammo = self.current_ammo.saturating_sub(1);
        }
    }

    /// Get display string for UI (e.g., "Plasma Cutter [3/5]")
    pub fn display(&self) -> String {
        if self.max_ammo < 0 {
            format!("{} [∞]", self.name)
        } else {
            format!("{} [{}/{}]", self.name, self.current_ammo, self.max_ammo)
        }
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self::default_weapon()
    }
}
