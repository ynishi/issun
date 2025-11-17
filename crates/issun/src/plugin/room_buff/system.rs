//! Buff system (orchestration)

use super::service::BuffService;
use super::types::{ActiveBuff, ActiveBuffs, BuffConfig};

/// Buff system
///
/// Manages buff state and orchestrates buff operations.
#[derive(crate::System, Debug)]
#[system(name = "buff_system")]
pub struct BuffSystem {
    service: BuffService,
}

impl BuffSystem {
    pub fn new() -> Self {
        Self {
            service: BuffService::new(),
        }
    }

    /// Apply a buff from configuration
    pub fn apply_buff(&self, buffs: &mut ActiveBuffs, config: BuffConfig) {
        let active_buff = ActiveBuff::new(config);
        buffs.add(active_buff);
    }

    /// Tick all buffs (called each turn)
    pub fn tick(&self, buffs: &mut ActiveBuffs) {
        // Decrement turn-based buffs
        for buff in &mut buffs.buffs {
            buff.tick();
        }

        // Remove expired buffs
        buffs.buffs.retain(|buff| !buff.is_expired());
    }

    /// Clear room-scoped buffs (called on room exit)
    pub fn clear_room_buffs(&self, buffs: &mut ActiveBuffs) {
        buffs.clear_room_buffs();
    }

    /// Remove all buffs
    pub fn clear_all(&self, buffs: &mut ActiveBuffs) {
        buffs.buffs.clear();
    }

    /// Get the service (for external use)
    pub fn service(&self) -> &BuffService {
        &self.service
    }
}

impl Default for BuffSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::room_buff::types::{BuffDuration, BuffEffect};

    #[test]
    fn test_apply_buff() {
        let system = BuffSystem::new();
        let mut buffs = ActiveBuffs::new();

        let config = BuffConfig {
            id: "test".to_string(),
            name: "Test Buff".to_string(),
            duration: BuffDuration::Turns(3),
            effect: BuffEffect::AttackBonus(5),
        };

        system.apply_buff(&mut buffs, config);
        assert_eq!(buffs.len(), 1);
    }

    #[test]
    fn test_tick() {
        let system = BuffSystem::new();
        let mut buffs = ActiveBuffs::new();

        let config = BuffConfig {
            id: "test".to_string(),
            name: "Test Buff".to_string(),
            duration: BuffDuration::Turns(2),
            effect: BuffEffect::AttackBonus(5),
        };

        system.apply_buff(&mut buffs, config);
        assert_eq!(buffs.buffs[0].remaining_turns, Some(2));

        system.tick(&mut buffs);
        assert_eq!(buffs.buffs[0].remaining_turns, Some(1));

        system.tick(&mut buffs);
        assert_eq!(buffs.len(), 0); // Expired and removed
    }

    #[test]
    fn test_clear_room_buffs() {
        let system = BuffSystem::new();
        let mut buffs = ActiveBuffs::new();

        // Room buff
        system.apply_buff(
            &mut buffs,
            BuffConfig {
                id: "room".to_string(),
                name: "Room Buff".to_string(),
                duration: BuffDuration::UntilRoomExit,
                effect: BuffEffect::AttackBonus(5),
            },
        );

        // Permanent buff
        system.apply_buff(
            &mut buffs,
            BuffConfig {
                id: "permanent".to_string(),
                name: "Permanent Buff".to_string(),
                duration: BuffDuration::Permanent,
                effect: BuffEffect::DefenseBonus(3),
            },
        );

        assert_eq!(buffs.len(), 2);

        system.clear_room_buffs(&mut buffs);
        assert_eq!(buffs.len(), 1); // Only permanent remains
    }
}
