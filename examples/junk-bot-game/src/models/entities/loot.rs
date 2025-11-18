//! Loot and item drop system

use super::{Rarity, Weapon, WeaponEffect};
use serde::{Deserialize, Serialize};

/// Effect that an item has when picked up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEffect {
    /// Weapon drop with attack power and ammo
    Weapon { attack: i32, ammo: i32 },
    /// Armor/Defense bonus (player only)
    Armor(i32),
    /// HP recovery consumable
    Consumable(i32),
    /// Ammo refill for equipped weapon
    Ammo(i32),
}

/// Loot item that can be dropped by enemies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItem {
    pub name: String,
    pub description: String,
    pub rarity: Rarity,
    pub effect: ItemEffect,
}

impl LootItem {
    /// Create a new loot item
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        rarity: Rarity,
        effect: ItemEffect,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            rarity,
            effect,
        }
    }

    /// Create a weapon loot
    pub fn weapon(name: impl Into<String>, attack: i32, ammo: i32, rarity: Rarity) -> Self {
        Self::new(
            name,
            format!("A weapon with {} attack", attack),
            rarity,
            ItemEffect::Weapon { attack, ammo },
        )
    }

    /// Create armor loot
    pub fn armor(name: impl Into<String>, defense: i32, rarity: Rarity) -> Self {
        Self::new(
            name,
            format!("+{} Defense", defense),
            rarity,
            ItemEffect::Armor(defense),
        )
    }

    /// Create consumable loot
    pub fn consumable(name: impl Into<String>, hp: i32, rarity: Rarity) -> Self {
        Self::new(
            name,
            format!("Restores {} HP", hp),
            rarity,
            ItemEffect::Consumable(hp),
        )
    }

    /// Create ammo loot
    pub fn ammo(amount: i32) -> Self {
        Self::new(
            "Ammo Pack",
            format!("+{} Ammo", amount),
            Rarity::Common,
            ItemEffect::Ammo(amount),
        )
    }

    /// Convert weapon ItemEffect to Weapon entity
    pub fn to_weapon(&self, effect: WeaponEffect) -> Option<Weapon> {
        if let ItemEffect::Weapon { attack, ammo } = self.effect {
            Some(Weapon {
                name: self.name.clone(),
                attack,
                max_ammo: ammo,
                current_ammo: ammo,
                effect,
            })
        } else {
            None
        }
    }
}

/// Generate random loot based on rarity weights
pub fn generate_random_loot() -> LootItem {
    use issun::prelude::LootService;
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Use LootService for weighted rarity selection
    let selected_rarity = LootService::select_rarity(&mut rng);

    // Generate item based on rarity
    match selected_rarity {
        Rarity::Common => {
            if rng.gen_bool(0.5) {
                LootItem::ammo(10)
            } else {
                LootItem::consumable("Med Kit", 20, Rarity::Common)
            }
        }
        Rarity::Uncommon => {
            if rng.gen_bool(0.6) {
                LootItem::weapon("Plasma Pistol", 12, 15, Rarity::Uncommon)
            } else {
                LootItem::armor("Light Armor", 3, Rarity::Uncommon)
            }
        }
        Rarity::Rare => {
            if rng.gen_bool(0.7) {
                LootItem::weapon("Plasma Rifle", 18, 20, Rarity::Rare)
            } else {
                LootItem::consumable("Advanced Med Kit", 50, Rarity::Rare)
            }
        }
        Rarity::Epic => LootItem::weapon("Shotgun", 25, 8, Rarity::Epic),
        Rarity::Legendary => LootItem::weapon("Sniper Rifle", 40, 5, Rarity::Legendary),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loot_creation() {
        let weapon = LootItem::weapon("Test Gun", 10, 20, Rarity::Common);
        assert_eq!(weapon.name, "Test Gun");
        assert_eq!(weapon.rarity, Rarity::Common);
    }

    #[test]
    fn test_random_loot_generation() {
        for _ in 0..10 {
            let loot = generate_random_loot();
            // Should successfully generate loot
            assert!(!loot.name.is_empty());
        }
    }
}
