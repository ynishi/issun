//! Asset system for ISSUN
//!
//! Assets represent game content data (enemies, items, cards, etc.)

/// Asset trait for game content data
///
/// Assets are typically const data or loaded from files.
/// Examples: EnemyAsset, ItemAsset, CardAsset
pub trait Asset: Send + Sync {
    /// Asset type name
    fn asset_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Asset; // Import derive macro

    // Test Asset derive macro
    #[derive(Asset)]
    struct EnemyAsset {
        name: &'static str,
        hp: i32,
        attack: i32,
    }

    #[test]
    fn test_derived_asset() {
        let enemy = EnemyAsset {
            name: "Goblin",
            hp: 30,
            attack: 5,
        };

        // Asset trait should be implemented
        assert!(enemy.asset_type().contains("EnemyAsset"));
    }

    #[derive(Asset)]
    struct ItemAsset {
        name: &'static str,
        value: i32,
    }

    #[test]
    fn test_multiple_assets() {
        let enemy = EnemyAsset {
            name: "Dragon",
            hp: 100,
            attack: 20,
        };

        let item = ItemAsset {
            name: "Sword",
            value: 50,
        };

        assert!(enemy.asset_type().contains("EnemyAsset"));
        assert!(item.asset_type().contains("ItemAsset"));
    }
}
