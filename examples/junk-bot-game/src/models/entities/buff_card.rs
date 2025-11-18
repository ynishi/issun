use super::Rarity;
use crate::assets::BUFF_CARDS;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Types of buffs that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffType {
    /// Increases attack by a fixed amount
    AttackUp(i32),
    /// Increases max HP by a fixed amount
    HpUp(i32),
    /// Increases drop rate by a multiplier (e.g., 1.0 = +100%)
    DropRateUp(f32),
    /// Increases critical hit rate by a percentage (e.g., 0.2 = +20%)
    CriticalUp(f32),
    /// Increases speed by a fixed amount
    SpeedUp(i32),
}

/// Buff card that provides stat boosts to the player or party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffCard {
    pub name: String,
    pub description: String,
    pub buff_type: BuffType,
    pub rarity: Rarity,
}

impl BuffCard {
    /// Create a new buff card
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        buff_type: BuffType,
        rarity: Rarity,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            buff_type,
            rarity,
        }
    }
}

/// Generate random buff cards from the asset pool
pub fn generate_random_cards(count: usize) -> Vec<BuffCard> {
    let mut rng = rand::thread_rng();
    let mut cards = Vec::with_capacity(count);

    for _ in 0..count {
        let asset = &BUFF_CARDS[rng.gen_range(0..BUFF_CARDS.len())];
        cards.push(BuffCard::new(
            asset.name,
            asset.description,
            asset.buff_type.clone(),
            asset.rarity,
        ));
    }

    cards
}
