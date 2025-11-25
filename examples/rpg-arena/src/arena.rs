//! Arena resource - main game state

use crate::combat_state::{CombatManager, CombatState};
use crate::fighter::Fighter;
use crate::item::{Inventory, Item};
use serde::{Deserialize, Serialize};

/// Arena game state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Arena {
    pub player: Fighter,
    pub enemy: Fighter,
    pub inventory: Inventory,
    pub combat: CombatManager,
    pub difficulty_multiplier: f32,
}

impl Arena {
    /// Create new arena with default settings
    pub fn new(max_hp: u32, max_slots: usize, allow_stacking: bool) -> Self {
        let player = Fighter::new("🧙 Player".to_string(), max_hp, 10);
        let enemy = Fighter::new("👹 Enemy".to_string(), 100, 8);

        let mut arena = Self {
            player,
            enemy,
            inventory: Inventory::new(max_slots, allow_stacking),
            combat: CombatManager::new(),
            difficulty_multiplier: 1.0,
        };

        // Add starting items
        arena.inventory.add_item(Item::hp_potion()).ok();
        arena.inventory.add_item(Item::hp_potion()).ok();
        arena.inventory.add_item(Item::attack_boost()).ok();

        arena
    }

    /// Start a new combat
    pub fn start_combat(&mut self) {
        self.combat.start_combat();
    }

    /// Player attacks enemy
    pub fn player_attack(&mut self) {
        if self.combat.state != CombatState::PlayerTurn {
            return;
        }

        let damage = self.player.calculate_damage(self.difficulty_multiplier);
        self.enemy.take_damage(damage);

        let message = format!(
            "🧙 Player attacks for {} damage! (Enemy HP: {}/{})",
            damage, self.enemy.current_hp, self.enemy.max_hp
        );
        self.combat.add_log(message);

        if !self.enemy.is_alive {
            self.combat.end_combat(true);
        } else {
            self.combat.next_turn();
        }
    }

    /// Enemy attacks player (AI)
    pub fn enemy_attack(&mut self) {
        if self.combat.state != CombatState::EnemyTurn {
            return;
        }

        let damage = self.enemy.calculate_damage(self.difficulty_multiplier);
        self.player.take_damage(damage);

        let message = format!(
            "👹 Enemy attacks for {} damage! (Player HP: {}/{})",
            damage, self.player.current_hp, self.player.max_hp
        );
        self.combat.add_log(message);

        if !self.player.is_alive {
            self.combat.end_combat(false);
        } else {
            self.combat.next_turn();
        }
    }

    /// Player uses an item
    pub fn use_item(&mut self, index: usize) -> Result<(), String> {
        if self.combat.state != CombatState::PlayerTurn {
            return Err("Not player's turn".to_string());
        }

        let item = self
            .inventory
            .use_item(index)
            .ok_or("Invalid item index".to_string())?;

        let message = match item.effect {
            crate::item::ItemEffect::HealHP(amount) => {
                self.player.heal(amount);
                format!(
                    "🧪 Used {} - Healed {} HP (Player HP: {}/{})",
                    item.name, amount, self.player.current_hp, self.player.max_hp
                )
            }
            crate::item::ItemEffect::BoostAttack(amount) => {
                self.player.base_attack += amount;
                format!(
                    "⚔️ Used {} - Attack +{} (Attack: {})",
                    item.name, amount, self.player.base_attack
                )
            }
        };

        self.combat.add_log(message);
        self.combat.next_turn();

        Ok(())
    }

    /// Reset arena for new game
    pub fn reset(&mut self, max_hp: u32, max_slots: usize, allow_stacking: bool) {
        *self = Self::new(max_hp, max_slots, allow_stacking);
    }

    /// Update difficulty multiplier from config
    pub fn update_difficulty(&mut self, multiplier: f32) {
        self.difficulty_multiplier = multiplier;
    }

    /// Check if combat is active
    pub fn is_combat_active(&self) -> bool {
        matches!(
            self.combat.state,
            CombatState::PlayerTurn | CombatState::EnemyTurn
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let arena = Arena::new(100, 10, true);
        assert_eq!(arena.player.max_hp, 100);
        assert_eq!(arena.enemy.max_hp, 100);
        assert_eq!(arena.inventory.max_slots, 10);
        assert!(arena.inventory.item_count() > 0); // Has starting items
    }

    #[test]
    fn test_combat_flow() {
        let mut arena = Arena::new(100, 10, true);
        arena.start_combat();

        assert_eq!(arena.combat.state, CombatState::PlayerTurn);

        // Player attacks
        arena.player_attack();
        assert_eq!(arena.combat.state, CombatState::EnemyTurn);
        assert!(arena.enemy.current_hp < arena.enemy.max_hp);

        // Enemy attacks
        arena.enemy_attack();
        assert_eq!(arena.combat.state, CombatState::PlayerTurn);
        assert!(arena.player.current_hp < arena.player.max_hp);
    }

    #[test]
    fn test_use_item() {
        let mut arena = Arena::new(100, 10, true);
        arena.start_combat();
        arena.player.take_damage(50);

        let initial_hp = arena.player.current_hp;

        // Use HP potion (should be at index 0)
        let result = arena.use_item(0);
        assert!(result.is_ok());
        assert!(arena.player.current_hp > initial_hp);
    }

    #[test]
    fn test_difficulty_multiplier() {
        let mut arena = Arena::new(100, 10, true);
        arena.update_difficulty(2.0);
        arena.start_combat();

        let initial_hp = arena.enemy.current_hp;
        arena.player_attack();

        let damage = initial_hp - arena.enemy.current_hp;
        assert_eq!(damage, 20); // base 10 * 2.0 difficulty
    }
}
