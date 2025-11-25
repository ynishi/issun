//! Item model - inventory items with effects

use serde::{Deserialize, Serialize};

/// Item effect types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemEffect {
    HealHP(u32),
    BoostAttack(u32),
}

/// An item in the inventory
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub effect: ItemEffect,
}

impl Item {
    pub fn hp_potion() -> Self {
        Self {
            name: "HP Potion".to_string(),
            effect: ItemEffect::HealHP(30),
        }
    }

    pub fn attack_boost() -> Self {
        Self {
            name: "Attack Boost".to_string(),
            effect: ItemEffect::BoostAttack(5),
        }
    }

    /// Get item icon for display
    pub fn icon(&self) -> &'static str {
        match self.effect {
            ItemEffect::HealHP(_) => "🧪",
            ItemEffect::BoostAttack(_) => "⚔️",
        }
    }

    /// Check if two items can stack
    pub fn can_stack_with(&self, other: &Item) -> bool {
        self.name == other.name && self.effect == other.effect
    }
}

/// Inventory system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<(Item, u32)>, // (item, count)
    pub max_slots: usize,
    pub allow_stacking: bool,
}

impl Inventory {
    pub fn new(max_slots: usize, allow_stacking: bool) -> Self {
        Self {
            items: Vec::new(),
            max_slots,
            allow_stacking,
        }
    }

    /// Add an item to inventory
    pub fn add_item(&mut self, item: Item) -> Result<(), String> {
        if self.allow_stacking {
            // Try to stack with existing item
            if let Some((_, count)) = self
                .items
                .iter_mut()
                .find(|(existing, _)| existing.can_stack_with(&item))
            {
                *count += 1;
                return Ok(());
            }
        }

        // Add as new slot
        if self.max_slots > 0 && self.items.len() >= self.max_slots {
            return Err("Inventory full".to_string());
        }

        self.items.push((item, 1));
        Ok(())
    }

    /// Use an item (remove one from stack)
    pub fn use_item(&mut self, index: usize) -> Option<Item> {
        if index >= self.items.len() {
            return None;
        }

        let (item, count) = &mut self.items[index];
        let result = item.clone();

        if *count > 1 {
            *count -= 1;
        } else {
            self.items.remove(index);
        }

        Some(result)
    }

    /// Get all items for display
    pub fn items(&self) -> &[(Item, u32)] {
        &self.items
    }

    /// Get current item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if inventory is full
    pub fn is_full(&self) -> bool {
        self.max_slots > 0 && self.items.len() >= self.max_slots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_stacking() {
        let mut inv = Inventory::new(5, true);

        inv.add_item(Item::hp_potion()).unwrap();
        inv.add_item(Item::hp_potion()).unwrap();

        assert_eq!(inv.item_count(), 1);
        assert_eq!(inv.items()[0].1, 2); // Count is 2
    }

    #[test]
    fn test_inventory_no_stacking() {
        let mut inv = Inventory::new(5, false);

        inv.add_item(Item::hp_potion()).unwrap();
        inv.add_item(Item::hp_potion()).unwrap();

        assert_eq!(inv.item_count(), 2);
        assert_eq!(inv.items()[0].1, 1);
        assert_eq!(inv.items()[1].1, 1);
    }

    #[test]
    fn test_inventory_full() {
        let mut inv = Inventory::new(2, false);

        inv.add_item(Item::hp_potion()).unwrap();
        inv.add_item(Item::attack_boost()).unwrap();

        let result = inv.add_item(Item::hp_potion());
        assert!(result.is_err());
        assert!(inv.is_full());
    }

    #[test]
    fn test_use_item() {
        let mut inv = Inventory::new(5, true);

        inv.add_item(Item::hp_potion()).unwrap();
        inv.add_item(Item::hp_potion()).unwrap();

        let item = inv.use_item(0).unwrap();
        assert_eq!(item.name, "HP Potion");
        assert_eq!(inv.items()[0].1, 1); // Count decreased

        inv.use_item(0).unwrap();
        assert_eq!(inv.item_count(), 0); // All used
    }
}
