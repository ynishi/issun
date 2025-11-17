//! Room buff plugin types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Buff configuration database
///
/// Register this as a Resource during game initialization.
#[derive(crate::Resource, Clone, Debug)]
pub struct RoomBuffDatabase {
    pub buffs: HashMap<String, BuffConfig>,
}

impl RoomBuffDatabase {
    pub fn new() -> Self {
        Self {
            buffs: HashMap::new(),
        }
    }

    pub fn with_buff(mut self, id: impl Into<String>, config: BuffConfig) -> Self {
        self.buffs.insert(id.into(), config);
        self
    }

    pub fn get(&self, id: &str) -> Option<&BuffConfig> {
        self.buffs.get(id)
    }
}

impl Default for RoomBuffDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Buff configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuffConfig {
    pub id: String,
    pub name: String,
    pub duration: BuffDuration,
    pub effect: BuffEffect,
}

/// Buff duration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BuffDuration {
    /// Permanent (until game ends)
    Permanent,
    /// Until room exit
    UntilRoomExit,
    /// N turns
    Turns(u32),
}

/// Buff effect types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BuffEffect {
    /// Attack bonus (+N)
    AttackBonus(i32),
    /// Defense bonus (+N)
    DefenseBonus(i32),
    /// HP regeneration per turn
    HpRegen(i32),
    /// Drop rate multiplier (x N)
    DropRateMultiplier(f32),
}

/// Active buffs state (stored in `ResourceContext`)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActiveBuffs {
    pub buffs: Vec<ActiveBuff>,
}

impl ActiveBuffs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, buff: ActiveBuff) {
        self.buffs.push(buff);
    }

    pub fn clear_room_buffs(&mut self) {
        self.buffs
            .retain(|buff| !matches!(buff.config.duration, BuffDuration::UntilRoomExit));
    }

    pub fn is_empty(&self) -> bool {
        self.buffs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffs.len()
    }
}

/// An active buff instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveBuff {
    pub config: BuffConfig,
    pub remaining_turns: Option<u32>,
}

impl ActiveBuff {
    pub fn new(config: BuffConfig) -> Self {
        let remaining_turns = match config.duration {
            BuffDuration::Turns(n) => Some(n),
            _ => None,
        };

        Self {
            config,
            remaining_turns,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.remaining_turns.map_or(false, |turns| turns == 0)
    }

    pub fn tick(&mut self) {
        if let Some(turns) = self.remaining_turns.as_mut() {
            *turns = turns.saturating_sub(1);
        }
    }
}
