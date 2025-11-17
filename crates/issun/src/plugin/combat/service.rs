//! Combat service for domain logic
//!
//! Provides centralized damage calculation and combat mechanics.
//! Follows Domain-Driven Design principles - combat logic as a service.

use super::types::Combatant;

/// Result of a damage application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageResult {
    /// Actual damage dealt after defense calculation
    pub actual_damage: i32,
    /// Whether the target died from this damage
    pub is_dead: bool,
    /// Whether damage was reduced by defense
    pub was_blocked: bool,
}

/// Combat service providing centralized combat calculations
///
/// This service handles all combat-related calculations, similar to
/// how a bank service handles money transfers. Ensures consistent
/// damage calculation, defense application, and HP management.
///
/// # Example
///
/// ```ignore
/// let service = CombatService::new();
/// let result = service.apply_damage(target, 50, Some(10));
/// assert_eq!(result.actual_damage, 40);
/// ```
#[derive(Debug, Clone, issun_macros::Service)]
#[service(name = "combat_service")]
pub struct CombatService {
    /// Minimum damage that can be dealt (even with high defense)
    min_damage: i32,
}

impl CombatService {
    /// Create a new combat service with default settings
    pub fn new() -> Self {
        Self { min_damage: 1 }
    }

    /// Create a combat service with custom minimum damage
    pub fn with_min_damage(min_damage: i32) -> Self {
        Self { min_damage }
    }

    /// Apply damage to a target (like a bank transfer)
    ///
    /// This is the core damage application logic. It:
    /// 1. Calculates actual damage based on defense
    /// 2. Applies damage to target
    /// 3. Returns detailed result
    ///
    /// # Arguments
    ///
    /// * `target` - The combatant receiving damage (mutable reference)
    /// * `base_damage` - Raw damage before defense calculation
    /// * `defense` - Optional defense value to reduce damage
    ///
    /// # Returns
    ///
    /// DamageResult with actual damage, death status, and block status
    pub fn apply_damage<C: Combatant + ?Sized>(
        &self,
        target: &mut C,
        base_damage: i32,
        defense: Option<i32>,
    ) -> DamageResult {
        let actual_damage = self.calculate_damage(base_damage, defense);
        let was_blocked = defense.is_some() && actual_damage < base_damage;

        target.take_damage(actual_damage);

        DamageResult {
            actual_damage,
            is_dead: !target.is_alive(),
            was_blocked,
        }
    }

    /// Calculate damage after defense (pure function)
    ///
    /// # Formula
    ///
    /// - With defense: max(base_damage - defense, min_damage)
    /// - Without defense: base_damage
    ///
    /// This ensures even high defense allows at least min_damage through.
    pub fn calculate_damage(&self, base_damage: i32, defense: Option<i32>) -> i32 {
        if let Some(def) = defense {
            (base_damage - def).max(self.min_damage)
        } else {
            base_damage
        }
    }

    /// Calculate attack damage from an attacker
    ///
    /// # Arguments
    ///
    /// * `attacker` - The combatant attacking (immutable reference)
    /// * `multiplier` - Damage multiplier (e.g., from room buffs, critical hits)
    ///
    /// # Returns
    ///
    /// Base damage (before defense calculation)
    pub fn calculate_attack_damage<C: Combatant + ?Sized>(&self, attacker: &C, multiplier: f32) -> i32 {
        (attacker.attack() as f32 * multiplier) as i32
    }

    /// Transfer HP from one combatant to another (vampire attacks)
    ///
    /// # Arguments
    ///
    /// * `from` - Source combatant (loses HP, mutable reference)
    /// * `to` - Target combatant (would gain HP, if heal is implemented)
    /// * `amount` - Amount to transfer
    ///
    /// # Returns
    ///
    /// Actual amount transferred (capped by source's current HP)
    ///
    /// Note: Currently only damages source. Heal functionality
    /// would require adding heal() to Combatant trait.
    pub fn transfer_hp<C1: Combatant + ?Sized, C2: Combatant + ?Sized>(
        &self,
        from: &mut C1,
        _to: &mut C2,
        amount: i32,
    ) -> i32 {
        let actual = amount.min(from.hp());
        from.take_damage(actual);
        // TODO: to.heal(actual) when heal() is added to Combatant
        actual
    }

    /// Apply damage with attacker and defender
    ///
    /// Convenience method that combines attack calculation and damage application.
    ///
    /// # Arguments
    ///
    /// * `attacker` - The attacking combatant (immutable reference)
    /// * `defender` - The defending combatant (mutable reference)
    /// * `multiplier` - Damage multiplier (1.0 = normal, 2.0 = double damage, etc.)
    ///
    /// # Returns
    ///
    /// DamageResult with actual damage and status
    pub fn apply_attack<A: Combatant + ?Sized, D: Combatant + ?Sized>(
        &self,
        attacker: &A,
        defender: &mut D,
        multiplier: f32,
    ) -> DamageResult {
        let base_damage = self.calculate_attack_damage(attacker, multiplier);
        let defense = defender.defense();
        self.apply_damage(defender, base_damage, defense)
    }
}

impl Default for CombatService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock combatant for testing
    struct MockCombatant {
        name: String,
        hp: i32,
        max_hp: i32,
        attack: i32,
        defense: Option<i32>,
    }

    impl Combatant for MockCombatant {
        fn name(&self) -> &str {
            &self.name
        }

        fn hp(&self) -> i32 {
            self.hp
        }

        fn max_hp(&self) -> i32 {
            self.max_hp
        }

        fn attack(&self) -> i32 {
            self.attack
        }

        fn defense(&self) -> Option<i32> {
            self.defense
        }

        fn take_damage(&mut self, damage: i32) {
            self.hp = (self.hp - damage).max(0);
        }
    }

    #[test]
    fn test_calculate_damage_no_defense() {
        let service = CombatService::new();
        assert_eq!(service.calculate_damage(50, None), 50);
        assert_eq!(service.calculate_damage(100, None), 100);
    }

    #[test]
    fn test_calculate_damage_with_defense() {
        let service = CombatService::new();
        assert_eq!(service.calculate_damage(50, Some(10)), 40);
        assert_eq!(service.calculate_damage(50, Some(30)), 20);
    }

    #[test]
    fn test_calculate_damage_minimum() {
        let service = CombatService::new();
        // Even with high defense, at least 1 damage gets through
        assert_eq!(service.calculate_damage(10, Some(50)), 1);
        assert_eq!(service.calculate_damage(5, Some(100)), 1);
    }

    #[test]
    fn test_apply_damage() {
        let service = CombatService::new();
        let mut target = MockCombatant {
            name: "Target".to_string(),
            hp: 100,
            max_hp: 100,
            attack: 10,
            defense: Some(5),
        };

        let result = service.apply_damage(&mut target, 30, Some(5));

        assert_eq!(result.actual_damage, 25);
        assert_eq!(result.was_blocked, true);
        assert_eq!(result.is_dead, false);
        assert_eq!(target.hp, 75);
    }

    #[test]
    fn test_apply_damage_lethal() {
        let service = CombatService::new();
        let mut target = MockCombatant {
            name: "Target".to_string(),
            hp: 20,
            max_hp: 100,
            attack: 10,
            defense: None,
        };

        let result = service.apply_damage(&mut target, 50, None);

        assert_eq!(result.actual_damage, 50);
        assert_eq!(result.was_blocked, false);
        assert_eq!(result.is_dead, true);
        assert_eq!(target.hp, 0);
    }

    #[test]
    fn test_calculate_attack_damage() {
        let service = CombatService::new();
        let attacker = MockCombatant {
            name: "Attacker".to_string(),
            hp: 100,
            max_hp: 100,
            attack: 50,
            defense: None,
        };

        assert_eq!(service.calculate_attack_damage(&attacker, 1.0), 50);
        assert_eq!(service.calculate_attack_damage(&attacker, 1.5), 75);
        assert_eq!(service.calculate_attack_damage(&attacker, 2.0), 100);
    }

    #[test]
    fn test_apply_attack() {
        let service = CombatService::new();
        let attacker = MockCombatant {
            name: "Attacker".to_string(),
            hp: 100,
            max_hp: 100,
            attack: 50,
            defense: None,
        };
        let mut defender = MockCombatant {
            name: "Defender".to_string(),
            hp: 100,
            max_hp: 100,
            attack: 30,
            defense: Some(10),
        };

        let result = service.apply_attack(&attacker, &mut defender, 1.0);

        // 50 attack - 10 defense = 40 damage
        assert_eq!(result.actual_damage, 40);
        assert_eq!(result.was_blocked, true);
        assert_eq!(defender.hp, 60);
    }

    #[test]
    fn test_transfer_hp() {
        let service = CombatService::new();
        let mut source = MockCombatant {
            name: "Source".to_string(),
            hp: 50,
            max_hp: 100,
            attack: 10,
            defense: None,
        };
        let mut target = MockCombatant {
            name: "Target".to_string(),
            hp: 30,
            max_hp: 100,
            attack: 10,
            defense: None,
        };

        let transferred = service.transfer_hp(&mut source, &mut target, 20);

        assert_eq!(transferred, 20);
        assert_eq!(source.hp, 30);
        // Note: target HP unchanged until heal() is implemented
    }

    #[test]
    fn test_transfer_hp_capped() {
        let service = CombatService::new();
        let mut source = MockCombatant {
            name: "Source".to_string(),
            hp: 10,
            max_hp: 100,
            attack: 10,
            defense: None,
        };
        let mut target = MockCombatant {
            name: "Target".to_string(),
            hp: 30,
            max_hp: 100,
            attack: 10,
            defense: None,
        };

        // Try to transfer 50, but only 10 HP available
        let transferred = service.transfer_hp(&mut source, &mut target, 50);

        assert_eq!(transferred, 10);
        assert_eq!(source.hp, 0);
    }
}
