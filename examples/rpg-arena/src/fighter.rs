//! Fighter model - represents a combatant in the arena

use serde::{Deserialize, Serialize};

/// A fighter in the arena
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fighter {
    pub name: String,
    pub max_hp: u32,
    pub current_hp: u32,
    pub base_attack: u32,
    pub is_alive: bool,
}

impl Fighter {
    pub fn new(name: String, max_hp: u32, base_attack: u32) -> Self {
        Self {
            name,
            max_hp,
            current_hp: max_hp,
            base_attack,
            is_alive: true,
        }
    }

    /// Take damage and update alive status
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.current_hp {
            self.current_hp = 0;
            self.is_alive = false;
        } else {
            self.current_hp -= damage;
        }
    }

    /// Heal HP (capped at max_hp)
    pub fn heal(&mut self, amount: u32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    /// Calculate actual damage with difficulty multiplier
    pub fn calculate_damage(&self, difficulty: f32) -> u32 {
        (self.base_attack as f32 * difficulty) as u32
    }

    /// Get HP percentage
    pub fn hp_percentage(&self) -> f32 {
        if self.max_hp == 0 {
            0.0
        } else {
            self.current_hp as f32 / self.max_hp as f32
        }
    }

    /// Get HP bar for display
    pub fn hp_bar(&self, width: usize) -> String {
        let filled = (self.hp_percentage() * width as f32) as usize;
        let empty = width.saturating_sub(filled);
        format!("{}{}",  "█".repeat(filled), "░".repeat(empty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fighter_creation() {
        let fighter = Fighter::new("Hero".to_string(), 100, 10);
        assert_eq!(fighter.name, "Hero");
        assert_eq!(fighter.max_hp, 100);
        assert_eq!(fighter.current_hp, 100);
        assert_eq!(fighter.base_attack, 10);
        assert!(fighter.is_alive);
    }

    #[test]
    fn test_take_damage() {
        let mut fighter = Fighter::new("Hero".to_string(), 100, 10);
        fighter.take_damage(30);
        assert_eq!(fighter.current_hp, 70);
        assert!(fighter.is_alive);

        fighter.take_damage(80);
        assert_eq!(fighter.current_hp, 0);
        assert!(!fighter.is_alive);
    }

    #[test]
    fn test_heal() {
        let mut fighter = Fighter::new("Hero".to_string(), 100, 10);
        fighter.take_damage(50);
        fighter.heal(30);
        assert_eq!(fighter.current_hp, 80);

        // Can't heal above max_hp
        fighter.heal(50);
        assert_eq!(fighter.current_hp, 100);
    }

    #[test]
    fn test_calculate_damage_with_difficulty() {
        let fighter = Fighter::new("Hero".to_string(), 100, 10);
        assert_eq!(fighter.calculate_damage(1.0), 10);
        assert_eq!(fighter.calculate_damage(2.0), 20);
        assert_eq!(fighter.calculate_damage(0.5), 5);
    }
}
