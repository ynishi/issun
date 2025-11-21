//! Hook trait for custom combat behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::events::BattleId;
use super::types::{CombatResult, Combatant};

/// Trait for custom combat behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., damage modifiers based on buffs)
/// - Direct resource modification (e.g., applying status effects, awarding XP)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait CombatHook: Send + Sync {
    /// Called before each combat turn
    ///
    /// Use this to apply turn-based effects (poison, regeneration, buffs, etc.)
    ///
    /// # Arguments
    ///
    /// * `battle_id` - Unique identifier for this battle
    /// * `turn` - Current turn number
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// `Ok(())` to continue turn, `Err(reason)` to cancel turn
    ///
    /// # Default
    ///
    /// Always allows turn to proceed
    async fn before_turn(
        &self,
        _battle_id: &BattleId,
        _turn: u32,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Calculate damage modifier for an attack
    ///
    /// Allows game-specific bonuses/penalties based on context.
    /// Examples: critical hits, elemental weaknesses, buff/debuff effects
    ///
    /// # Arguments
    ///
    /// * `battle_id` - Unique identifier for this battle
    /// * `attacker` - The combatant attacking
    /// * `defender` - The combatant defending
    /// * `base_multiplier` - Base damage multiplier (e.g., from room buffs)
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Final damage multiplier (1.0 = normal, 2.0 = double damage, etc.)
    ///
    /// # Default
    ///
    /// Returns base_multiplier unchanged
    async fn calculate_damage_modifier(
        &self,
        _battle_id: &BattleId,
        _attacker: &dyn Combatant,
        _defender: &dyn Combatant,
        base_multiplier: f32,
        _resources: &ResourceContext,
    ) -> f32 {
        base_multiplier
    }

    /// Process a single combat turn
    ///
    /// **This is the main hook for game-specific combat logic.**
    ///
    /// The hook should:
    /// 1. Determine who attacks whom
    /// 2. Calculate and apply damage
    /// 3. Return log entries describing what happened
    ///
    /// # Arguments
    ///
    /// * `battle_id` - Unique identifier for this battle
    /// * `turn` - Current turn number
    /// * `resources` - Access to game resources for reading combat state and modifying combatants
    ///
    /// # Returns
    ///
    /// Vector of log messages describing the turn's events
    ///
    /// # Default
    ///
    /// Returns empty log (no combat actions)
    async fn process_turn(
        &self,
        _battle_id: &BattleId,
        _turn: u32,
        _resources: &mut ResourceContext,
    ) -> Vec<String> {
        Vec::new()
    }

    /// Called after each combat turn
    ///
    /// Use this for logging, statistics tracking, or side effects.
    ///
    /// # Arguments
    ///
    /// * `battle_id` - Unique identifier for this battle
    /// * `turn` - Current turn number
    /// * `log_entries` - Log entries from this turn
    /// * `resources` - Access to game resources for modification
    async fn after_turn(
        &self,
        _battle_id: &BattleId,
        _turn: u32,
        _log_entries: &[String],
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when combat ends
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets the combat result and updates other resources.
    /// For example:
    /// - RPG: Award XP, loot, achievements
    /// - Strategy: Update campaign progress, unlock units
    /// - Roguelike: Grant permanent upgrades
    ///
    /// # Arguments
    ///
    /// * `battle_id` - Unique identifier for this battle
    /// * `result` - Combat result (Victory, Defeat, or Ongoing)
    /// * `total_turns` - Total number of turns in this battle
    /// * `score` - Total score accumulated
    /// * `resources` - Access to game resources for modification
    async fn on_combat_ended(
        &self,
        _battle_id: &BattleId,
        _result: &CombatResult,
        _total_turns: u32,
        _score: u32,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultCombatHook;

#[async_trait]
impl CombatHook for DefaultCombatHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultCombatHook;
        let battle_id = "test_battle".to_string();
        let resources = ResourceContext::new();

        // Should not panic
        let result = hook.before_turn(&battle_id, 1, &resources).await;
        assert!(result.is_ok());

        let multiplier = hook
            .calculate_damage_modifier(&battle_id, &MockCombatant, &MockCombatant, 1.5, &resources)
            .await;
        assert_eq!(multiplier, 1.5);

        let mut resources = ResourceContext::new();
        hook.after_turn(&battle_id, 1, &[], &mut resources).await;
        hook.on_combat_ended(&battle_id, &CombatResult::Victory, 5, 100, &mut resources)
            .await;
    }

    // Mock combatant for testing
    struct MockCombatant;
    impl Combatant for MockCombatant {
        fn name(&self) -> &str {
            "Mock"
        }
        fn hp(&self) -> i32 {
            100
        }
        fn max_hp(&self) -> i32 {
            100
        }
        fn attack(&self) -> i32 {
            10
        }
        fn take_damage(&mut self, _damage: i32) {}
    }
}
