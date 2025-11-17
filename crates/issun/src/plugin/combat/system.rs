//! Combat system implementation
//!
//! Core turn-based combat processing system.

use super::service::CombatService;
use super::{CombatLogEntry, CombatResult, Combatant, TurnBasedCombatConfig};

/// Core combat system
///
/// Generic over party members (P) and enemies (E) that implement Combatant
///
/// This demonstrates the System pattern for Application Logic.
/// CombatSystem orchestrates combat flow using CombatService.
#[derive(Debug, Clone, issun_macros::System)]
#[system(name = "combat_system")]
pub struct CombatSystem {
    turn_count: u32,
    log: Vec<CombatLogEntry>,
    config: TurnBasedCombatConfig,
    score: u32,
    combat_service: CombatService,
}

impl CombatSystem {
    pub fn new(config: TurnBasedCombatConfig) -> Self {
        Self {
            turn_count: 0,
            log: Vec::new(),
            config,
            score: 0,
            combat_service: CombatService::new(),
        }
    }

    /// Get current turn count
    pub fn turn_count(&self) -> u32 {
        self.turn_count
    }

    /// Get combat log
    pub fn log(&self) -> &[CombatLogEntry] {
        &self.log
    }

    /// Get accumulated score
    pub fn score(&self) -> u32 {
        self.score
    }

    /// Add log entry
    pub fn add_log(&mut self, message: String) {
        if !self.config.enable_log {
            return;
        }

        self.log.push(CombatLogEntry {
            turn: self.turn_count,
            message,
        });

        // Trim log if exceeds max
        if self.log.len() > self.config.max_log_entries {
            self.log
                .drain(0..self.log.len() - self.config.max_log_entries);
        }
    }

    /// Process a full combat turn with trait objects (for heterogeneous parties)
    ///
    /// # Arguments
    ///
    /// * `party` - Mutable slice of party member trait objects
    /// * `enemies` - Mutable slice of enemy trait objects
    /// * `damage_multiplier` - Multiplier for enemy damage (e.g., room buffs)
    /// * `per_turn_damage` - Damage applied to all party members per turn (e.g., poison)
    ///
    /// Returns CombatResult indicating Victory, Defeat, or Ongoing
    pub fn process_turn_dyn(
        &mut self,
        party: &mut [&mut dyn Combatant],
        enemies: &mut [&mut dyn Combatant],
        damage_multiplier: f32,
        per_turn_damage: i32,
    ) -> CombatResult {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        self.turn_count += 1;
        self.add_log(format!("--- Turn {} ---", self.turn_count));

        // Party attacks enemies (sequential targeting)
        for attacker_ref in party.iter_mut().filter(|p| p.is_alive()) {
            if let Some(enemy_ref) = enemies.iter_mut().find(|e| e.is_alive()) {
                let attacker_name = attacker_ref.name().to_string();
                let enemy_name = enemy_ref.name().to_string();

                // Use CombatService for damage calculation (with defense)
                let attacker_combatant: &mut dyn Combatant = *attacker_ref;
                let enemy_combatant: &mut dyn Combatant = *enemy_ref;
                let result =
                    self.combat_service
                        .apply_attack(attacker_combatant, enemy_combatant, 1.0);

                self.add_log(format!(
                    "{} attacks {} for {} damage!",
                    attacker_name, enemy_name, result.actual_damage
                ));

                if result.is_dead {
                    self.add_log(format!("{} defeated!", enemy_name));
                    self.score += self.config.score_per_enemy;
                }
            }
        }

        // Enemies attack party (random targeting)
        for enemy_ref in enemies.iter_mut().filter(|e| e.is_alive()) {
            // Build list of alive party members for targeting
            let alive_party_indices: Vec<usize> = party
                .iter()
                .enumerate()
                .filter(|(_, p)| p.is_alive())
                .map(|(i, _)| i)
                .collect();

            if alive_party_indices.is_empty() {
                break;
            }

            let enemy_name = enemy_ref.name().to_string();

            // Random target
            let target_idx = rng.gen_range(0..alive_party_indices.len());
            let party_idx = alive_party_indices[target_idx];

            let target_name = party[party_idx].name().to_string();

            // Use CombatService for damage calculation (with multiplier and defense)
            // enemy_ref is &mut &mut dyn Combatant, *enemy_ref gives &mut dyn Combatant
            // party[idx] already gives &mut dyn Combatant (no deref needed)
            let enemy_combatant: &mut dyn Combatant = *enemy_ref;
            let target_combatant: &mut dyn Combatant = party[party_idx];
            let result = self.combat_service.apply_attack(
                enemy_combatant,
                target_combatant,
                damage_multiplier,
            );

            self.add_log(format!(
                "{} attacks {} for {} damage!",
                enemy_name, target_name, result.actual_damage
            ));
        }

        // Apply per-turn damage (e.g., contamination)
        if per_turn_damage > 0 {
            for member_ref in party.iter_mut().filter(|p| p.is_alive()) {
                // Per-turn damage ignores defense
                let member_combatant: &mut dyn Combatant = *member_ref;
                let result =
                    self.combat_service
                        .apply_damage(member_combatant, per_turn_damage, None);
                self.add_log(format!(
                    "☢️ Contamination damages {} for {} HP!",
                    member_combatant.name(),
                    result.actual_damage
                ));
            }
        }

        // Check win/lose conditions
        let all_enemies_dead = enemies.iter().all(|e| !e.is_alive());
        let all_party_dead = party.iter().all(|p| !p.is_alive());

        if all_enemies_dead {
            CombatResult::Victory
        } else if all_party_dead {
            self.add_log("Your party has been defeated...".to_string());
            CombatResult::Defeat
        } else {
            CombatResult::Ongoing
        }
    }

    /// Process a full combat turn
    ///
    /// # Arguments
    ///
    /// * `party` - Mutable slice of party members
    /// * `enemies` - Mutable slice of enemies
    /// * `damage_multiplier` - Multiplier for enemy damage (e.g., room buffs)
    /// * `per_turn_damage` - Damage applied to all party members per turn (e.g., poison)
    ///
    /// Returns CombatResult indicating Victory, Defeat, or Ongoing
    pub fn process_turn<P, E>(
        &mut self,
        party: &mut [P],
        enemies: &mut [E],
        damage_multiplier: f32,
        per_turn_damage: i32,
    ) -> CombatResult
    where
        P: Combatant,
        E: Combatant,
    {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        self.turn_count += 1;
        self.add_log(format!("--- Turn {} ---", self.turn_count));

        // Party attacks enemies (sequential targeting)
        for attacker in party.iter_mut().filter(|p| p.is_alive()) {
            if let Some(enemy) = enemies.iter_mut().find(|e| e.is_alive()) {
                let damage = attacker.attack();
                let attacker_name = attacker.name().to_string();
                let enemy_name = enemy.name().to_string();

                enemy.take_damage(damage);
                self.add_log(format!(
                    "{} attacks {} for {} damage!",
                    attacker_name, enemy_name, damage
                ));

                if !enemy.is_alive() {
                    self.add_log(format!("{} defeated!", enemy_name));
                    self.score += self.config.score_per_enemy;
                }
            }
        }

        // Enemies attack party (random targeting)
        let alive_enemies: Vec<(String, i32)> = enemies
            .iter()
            .filter(|e| e.is_alive())
            .map(|e| (e.name().to_string(), e.attack()))
            .collect();

        // Build list of alive party member names
        let mut alive_party_names: Vec<String> = party
            .iter()
            .filter(|p| p.is_alive())
            .map(|p| p.name().to_string())
            .collect();

        for (enemy_name, base_damage) in alive_enemies {
            if alive_party_names.is_empty() {
                break;
            }

            // Apply damage multiplier
            let damage = (base_damage as f32 * damage_multiplier) as i32;

            // Random target
            let target_idx = rng.gen_range(0..alive_party_names.len());
            let target_name = alive_party_names[target_idx].clone();

            // Find and damage the target
            if let Some(target) = party.iter_mut().find(|p| p.name() == target_name) {
                target.take_damage(damage);
                self.add_log(format!(
                    "{} attacks {} for {} damage!",
                    enemy_name, target_name, damage
                ));

                if !target.is_alive() {
                    alive_party_names.retain(|n| n != &target_name);
                }
            }
        }

        // Apply per-turn damage (e.g., contamination)
        if per_turn_damage > 0 {
            for member in party.iter_mut().filter(|p| p.is_alive()) {
                member.take_damage(per_turn_damage);
                self.add_log(format!(
                    "☢️ Contamination damages {} for {} HP!",
                    member.name(),
                    per_turn_damage
                ));
            }
        }

        // Check win/lose conditions
        let all_enemies_dead = enemies.iter().all(|e| !e.is_alive());
        let all_party_dead = party.iter().all(|p| !p.is_alive());

        if all_enemies_dead {
            CombatResult::Victory
        } else if all_party_dead {
            self.add_log("Your party has been defeated...".to_string());
            CombatResult::Defeat
        } else {
            CombatResult::Ongoing
        }
    }
}
