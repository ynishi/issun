//! Buff service (pure logic)

use super::types::{ActiveBuffs, BuffEffect};

/// Buff service
///
/// Provides pure functions for buff calculations.
/// No state management - only calculations.
#[derive(crate::Service, Debug, Clone)]
#[service(name = "buff_service")]
pub struct BuffService;

impl BuffService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate total attack bonus from active buffs
    pub fn calculate_attack_bonus(&self, buffs: &ActiveBuffs) -> i32 {
        buffs
            .buffs
            .iter()
            .filter_map(|buff| match buff.config.effect {
                BuffEffect::AttackBonus(bonus) => Some(bonus),
                _ => None,
            })
            .sum()
    }

    /// Calculate total defense bonus from active buffs
    pub fn calculate_defense_bonus(&self, buffs: &ActiveBuffs) -> i32 {
        buffs
            .buffs
            .iter()
            .filter_map(|buff| match buff.config.effect {
                BuffEffect::DefenseBonus(bonus) => Some(bonus),
                _ => None,
            })
            .sum()
    }

    /// Calculate HP regeneration per turn
    pub fn calculate_hp_regen(&self, buffs: &ActiveBuffs) -> i32 {
        buffs
            .buffs
            .iter()
            .filter_map(|buff| match buff.config.effect {
                BuffEffect::HpRegen(regen) => Some(regen),
                _ => None,
            })
            .sum()
    }

    /// Calculate drop rate multiplier
    pub fn calculate_drop_rate_multiplier(&self, buffs: &ActiveBuffs) -> f32 {
        buffs
            .buffs
            .iter()
            .filter_map(|buff| match buff.config.effect {
                BuffEffect::DropRateMultiplier(mult) => Some(mult),
                _ => None,
            })
            .product::<f32>()
            .max(1.0) // At least 1.0x
    }

    /// Check if a specific buff effect is active
    pub fn has_effect(&self, buffs: &ActiveBuffs, effect_type: &BuffEffect) -> bool {
        buffs
            .buffs
            .iter()
            .any(|buff| std::mem::discriminant(&buff.config.effect) == std::mem::discriminant(effect_type))
    }
}

impl Default for BuffService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::room_buff::types::{ActiveBuff, BuffConfig, BuffDuration};

    #[test]
    fn test_calculate_attack_bonus() {
        let service = BuffService::new();
        let mut buffs = ActiveBuffs::new();

        buffs.add(ActiveBuff::new(BuffConfig {
            id: "buff1".to_string(),
            name: "Attack Up".to_string(),
            duration: BuffDuration::Permanent,
            effect: BuffEffect::AttackBonus(5),
        }));

        buffs.add(ActiveBuff::new(BuffConfig {
            id: "buff2".to_string(),
            name: "Attack Up 2".to_string(),
            duration: BuffDuration::Turns(3),
            effect: BuffEffect::AttackBonus(3),
        }));

        assert_eq!(service.calculate_attack_bonus(&buffs), 8);
    }

    #[test]
    fn test_calculate_drop_rate_multiplier() {
        let service = BuffService::new();
        let mut buffs = ActiveBuffs::new();

        buffs.add(ActiveBuff::new(BuffConfig {
            id: "lucky".to_string(),
            name: "Lucky".to_string(),
            duration: BuffDuration::UntilRoomExit,
            effect: BuffEffect::DropRateMultiplier(2.0),
        }));

        assert_eq!(service.calculate_drop_rate_multiplier(&buffs), 2.0);
    }
}
