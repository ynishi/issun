//! Buff system (orchestration)

use super::service::BuffService;
use super::types::{ActiveBuff, ActiveBuffs, BuffConfig};
use crate::context::ResourceContext;

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
    pub async fn apply_buff(&self, resources: &mut ResourceContext, config: BuffConfig) {
        let mut buffs = resources
            .get_mut::<ActiveBuffs>()
            .await
            .expect("ActiveBuffs not registered in ResourceContext");
        let active_buff = ActiveBuff::new(config);
        buffs.add(active_buff);
    }

    /// Tick all buffs (called each turn)
    pub async fn tick(&self, resources: &mut ResourceContext) {
        let mut buffs = resources
            .get_mut::<ActiveBuffs>()
            .await
            .expect("ActiveBuffs not registered in ResourceContext");
        // Decrement turn-based buffs
        for buff in &mut buffs.buffs {
            buff.tick();
        }

        // Remove expired buffs
        buffs.buffs.retain(|buff| !buff.is_expired());
    }

    /// Clear room-scoped buffs (called on room exit)
    pub async fn clear_room_buffs(&self, resources: &mut ResourceContext) {
        let mut buffs = resources
            .get_mut::<ActiveBuffs>()
            .await
            .expect("ActiveBuffs not registered in ResourceContext");
        buffs.clear_room_buffs();
    }

    /// Remove all buffs
    pub async fn clear_all(&self, resources: &mut ResourceContext) {
        let mut buffs = resources
            .get_mut::<ActiveBuffs>()
            .await
            .expect("ActiveBuffs not registered in ResourceContext");
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
    use crate::context::ResourceContext;
    use crate::plugin::room_buff::types::{BuffDuration, BuffEffect};

    fn context_with_buffs() -> ResourceContext {
        let mut resources = ResourceContext::new();
        resources.insert(ActiveBuffs::new());
        resources
    }

    #[tokio::test]
    async fn test_apply_buff() {
        let system = BuffSystem::new();
        let mut resources = context_with_buffs();
        let config = BuffConfig {
            id: "test".to_string(),
            name: "Test Buff".to_string(),
            duration: BuffDuration::Turns(3),
            effect: BuffEffect::AttackBonus(5),
        };

        system.apply_buff(&mut resources, config).await;
        let buffs = resources.get::<ActiveBuffs>().await.unwrap();
        assert_eq!(buffs.len(), 1);
    }

    #[tokio::test]
    async fn test_tick() {
        let system = BuffSystem::new();
        let mut resources = context_with_buffs();

        let config = BuffConfig {
            id: "test".to_string(),
            name: "Test Buff".to_string(),
            duration: BuffDuration::Turns(2),
            effect: BuffEffect::AttackBonus(5),
        };

        system.apply_buff(&mut resources, config).await;
        {
            let buffs = resources.get::<ActiveBuffs>().await.unwrap();
            assert_eq!(buffs.buffs[0].remaining_turns, Some(2));
        }

        system.tick(&mut resources).await;
        {
            let buffs = resources.get::<ActiveBuffs>().await.unwrap();
            assert_eq!(buffs.buffs[0].remaining_turns, Some(1));
        }

        system.tick(&mut resources).await;
        let buffs = resources.get::<ActiveBuffs>().await.unwrap();
        assert_eq!(buffs.len(), 0); // Expired and removed
    }

    #[tokio::test]
    async fn test_clear_room_buffs() {
        let system = BuffSystem::new();
        let mut resources = context_with_buffs();

        // Room buff
        system
            .apply_buff(
                &mut resources,
                BuffConfig {
                    id: "room".to_string(),
                    name: "Room Buff".to_string(),
                    duration: BuffDuration::UntilRoomExit,
                    effect: BuffEffect::AttackBonus(5),
                },
            )
            .await;

        // Permanent buff
        system
            .apply_buff(
                &mut resources,
                BuffConfig {
                    id: "permanent".to_string(),
                    name: "Permanent Buff".to_string(),
                    duration: BuffDuration::Permanent,
                    effect: BuffEffect::DefenseBonus(3),
                },
            )
            .await;

        {
            let buffs = resources.get::<ActiveBuffs>().await.unwrap();
            assert_eq!(buffs.len(), 2);
        }

        system.clear_room_buffs(&mut resources).await;
        let buffs = resources.get::<ActiveBuffs>().await.unwrap();
        assert_eq!(buffs.len(), 1); // Only permanent remains
    }
}
