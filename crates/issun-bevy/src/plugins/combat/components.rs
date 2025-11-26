//! Combat components for the combat system

use bevy::prelude::*;
use std::collections::HashMap;

// =============================================================================
// Combatant Components
// =============================================================================

/// Combatant component (name only)
///
/// ⚠️ CRITICAL: Must have #[derive(Reflect)] and #[reflect(Component)]!
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Combatant {
    pub name: String,
}

/// Health component (independent HP management)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }
}

/// Attack component (optional)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Attack {
    pub power: i32,
}

/// Defense component (optional)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Defense {
    pub value: i32,
}

/// Stable identifier for replay (does not change between runs)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct UniqueId(pub String);

// =============================================================================
// CombatSession Components
// =============================================================================

/// Combat session component
///
/// Holds the state of a single combat
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatSession {
    pub battle_id: String, // For identification (UI, logs)
    pub turn_count: u32,
    pub score: u32,
}

impl CombatSession {
    pub fn new(battle_id: String) -> Self {
        Self {
            battle_id,
            turn_count: 0,
            score: 0,
        }
    }
}

/// Combat participants list
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatParticipants {
    pub entities: Vec<Entity>,
}

impl CombatParticipants {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self { entities }
    }

    /// Remove zombie entities from participant list
    pub fn cleanup_zombies<F>(&mut self, is_alive: F)
    where
        F: Fn(Entity) -> bool,
    {
        self.entities.retain(|e| is_alive(*e));
    }
}

/// Combat log
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatLog {
    pub entries: Vec<CombatLogEntry>,
    pub max_entries: usize,
}

#[derive(Clone, Reflect)]
pub struct CombatLogEntry {
    pub turn: u32,
    pub message: String,
}

impl CombatLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add_entry(&mut self, turn: u32, message: String) {
        self.entries.push(CombatLogEntry { turn, message });

        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
}

/// Replay recorder component (attach to CombatSession)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplayRecorder {
    pub commands: Vec<RecordedCommand>,
    pub is_recording: bool,
}

impl ReplayRecorder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            is_recording: true,
        }
    }
}

impl Default for ReplayRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Recorded command with timestamp
#[derive(Clone, Reflect)]
pub struct RecordedCommand {
    pub frame: u32, // App update frame number
    pub command: CommandType,
}

#[derive(Clone, Reflect)]
pub enum CommandType {
    CombatStart {
        battle_id: String,
        participants: Vec<String>, // ⚠️ UniqueId, not Entity!
    },
    TurnAdvance {
        combat_id: String, // ⚠️ UniqueId, not Entity!
    },
    Damage {
        attacker_id: String, // ⚠️ UniqueId, not Entity!
        target_id: String,   // ⚠️ UniqueId, not Entity!
        base_damage: i32,
    },
    CombatEnd {
        combat_id: String, // ⚠️ UniqueId, not Entity!
    },
}

/// Per-combat seeded RNG for deterministic replay
///
/// ⚠️ CRITICAL: RNG must be per-combat to support parallel combats
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatSessionRng {
    pub seed: u64,
    // Note: StdRng doesn't implement Reflect, so we can't store it directly
    // Instead, we'll regenerate it when needed from the seed
}

impl CombatSessionRng {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate a random number in the given range
    /// For now, we'll use a simple deterministic approach
    /// TODO: Replace with proper StdRng when needed
    pub fn gen_range(&mut self, range: std::ops::Range<i32>) -> i32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        let value = hasher.finish();

        // Simple modulo to get value in range
        let range_size = (range.end - range.start) as u64;
        let result = range.start + ((value % range_size) as i32);

        // Update seed for next call
        self.seed = value;

        result
    }
}

// =============================================================================
// Resources
// =============================================================================

/// Combat system configuration (global)
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct CombatConfig {
    pub enable_log: bool,
    pub max_log_entries: usize,
    pub min_damage: i32, // Minimum guaranteed damage
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            enable_log: true,
            max_log_entries: 100,
            min_damage: 1,
        }
    }
}

/// Entity ID mapping for replay
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ReplayEntityMap {
    pub id_to_entity: HashMap<String, Entity>, // UniqueId → Entity
}

impl ReplayEntityMap {
    pub fn register(&mut self, unique_id: String, entity: Entity) {
        self.id_to_entity.insert(unique_id, entity);
    }

    pub fn get(&self, unique_id: &str) -> Option<Entity> {
        self.id_to_entity.get(unique_id).copied()
    }
}

/// Global frame counter for replay
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct FrameCount(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_is_alive() {
        let health = Health::new(100);
        assert!(health.is_alive());

        let mut dead_health = Health::new(100);
        dead_health.current = 0;
        assert!(!dead_health.is_alive());
    }

    #[test]
    fn test_health_take_damage() {
        let mut health = Health::new(100);
        health.take_damage(30);
        assert_eq!(health.current, 70);

        health.take_damage(80);
        assert_eq!(health.current, 0); // Doesn't go below 0
    }

    #[test]
    fn test_health_heal() {
        let mut health = Health::new(100);
        health.current = 50;
        health.heal(30);
        assert_eq!(health.current, 80);

        health.heal(50);
        assert_eq!(health.current, 100); // Doesn't exceed max
    }

    #[test]
    fn test_combat_log() {
        let mut log = CombatLog::new(3);
        log.add_entry(1, "Attack!".to_string());
        log.add_entry(2, "Defend!".to_string());
        log.add_entry(3, "Victory!".to_string());
        assert_eq!(log.entries.len(), 3);

        log.add_entry(4, "Extra".to_string());
        assert_eq!(log.entries.len(), 3); // Max entries enforced
        assert_eq!(log.entries[0].message, "Defend!"); // First entry removed
    }

    #[test]
    fn test_participants_cleanup() {
        let entity1 = Entity::from_bits(1);
        let entity2 = Entity::from_bits(2);
        let entity3 = Entity::from_bits(3);

        let mut participants = CombatParticipants::new(vec![entity1, entity2, entity3]);

        // Remove entity2 (simulate despawned entity)
        participants.cleanup_zombies(|e| e != entity2);

        assert_eq!(participants.entities.len(), 2);
        assert!(participants.entities.contains(&entity1));
        assert!(!participants.entities.contains(&entity2));
        assert!(participants.entities.contains(&entity3));
    }
}
