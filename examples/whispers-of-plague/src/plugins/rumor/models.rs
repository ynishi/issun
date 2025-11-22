use crate::models::GameMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type RumorId = String;

/// Rumor definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rumor {
    pub id: RumorId,
    pub name: String,
    pub description: String,
    pub effect: RumorEffect,
    pub initial_credibility: f32,
    pub mode_filter: Option<GameMode>,
}

/// Rumor effect types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RumorEffect {
    IncreasePanic(f32),
    DecreasePanic(f32),
    PromoteMigration { rate: f32 },
    PromoteIsolation { panic_reduction: f32 },
}

/// Registry of available rumors (Resource)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RumorRegistry {
    rumors: HashMap<RumorId, Rumor>,
}

impl RumorRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        // Plague mode rumors
        registry.add(Rumor {
            id: "fear_mongering".into(),
            name: "Government Conspiracy".into(),
            description: "Spread fear about cover-ups".into(),
            effect: RumorEffect::IncreasePanic(0.2),
            initial_credibility: 1.0,
            mode_filter: Some(GameMode::Plague),
        });

        registry.add(Rumor {
            id: "false_cure".into(),
            name: "Miracle Cure Discovered".into(),
            description: "False hope leads to dangerous behavior".into(),
            effect: RumorEffect::DecreasePanic(0.3),
            initial_credibility: 1.0,
            mode_filter: Some(GameMode::Plague),
        });

        registry.add(Rumor {
            id: "migration_trap".into(),
            name: "Safe Zone Identified".into(),
            description: "Direct people to infected area".into(),
            effect: RumorEffect::PromoteMigration { rate: 0.1 },
            initial_credibility: 1.0,
            mode_filter: Some(GameMode::Plague),
        });

        // Savior mode rumors
        registry.add(Rumor {
            id: "quarantine_success".into(),
            name: "Quarantine Works".into(),
            description: "Encourage isolation".into(),
            effect: RumorEffect::PromoteIsolation {
                panic_reduction: 0.15,
            },
            initial_credibility: 1.0,
            mode_filter: Some(GameMode::Savior),
        });

        registry.add(Rumor {
            id: "hope_message".into(),
            name: "Recovery Stories".into(),
            description: "Survivor stories reduce panic".into(),
            effect: RumorEffect::DecreasePanic(0.2),
            initial_credibility: 1.0,
            mode_filter: Some(GameMode::Savior),
        });

        registry.add(Rumor {
            id: "evacuation_order".into(),
            name: "Evacuation Order".into(),
            description: "Move away from infected zones".into(),
            effect: RumorEffect::PromoteMigration { rate: 0.15 },
            initial_credibility: 1.0,
            mode_filter: Some(GameMode::Savior),
        });

        registry
    }

    pub fn add(&mut self, rumor: Rumor) {
        self.rumors.insert(rumor.id.clone(), rumor);
    }

    pub fn get(&self, id: &RumorId) -> Option<&Rumor> {
        self.rumors.get(id)
    }

    pub fn get_by_mode(&self, mode: GameMode) -> Vec<&Rumor> {
        self.rumors
            .values()
            .filter(|r| r.mode_filter.is_none() || r.mode_filter == Some(mode))
            .collect()
    }

    pub fn all(&self) -> Vec<&Rumor> {
        self.rumors.values().collect()
    }
}

/// Active rumor tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRumor {
    pub rumor_id: RumorId,
    pub credibility: f32,
    pub turns_active: u32,
}

/// Runtime state for rumors
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RumorState {
    pub active_rumors: HashMap<RumorId, ActiveRumor>,
    pub rumor_history: Vec<String>,
}

impl RumorState {
    pub fn activate(&mut self, rumor_id: RumorId, initial_credibility: f32) {
        self.active_rumors.insert(
            rumor_id.clone(),
            ActiveRumor {
                rumor_id,
                credibility: initial_credibility,
                turns_active: 0,
            },
        );
    }

    pub fn get_active(&self, rumor_id: &RumorId) -> Option<&ActiveRumor> {
        self.active_rumors.get(rumor_id)
    }

    pub fn decay_all(&mut self, decay_rate: f32) {
        let to_remove: Vec<RumorId> = self
            .active_rumors
            .iter_mut()
            .filter_map(|(id, active)| {
                active.credibility = (active.credibility * decay_rate).max(0.0);
                active.turns_active += 1;
                if active.credibility < 0.1 {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        for id in to_remove {
            self.active_rumors.remove(&id);
        }
    }

    pub fn record_history(&mut self, message: String) {
        self.rumor_history.insert(0, message);
        self.rumor_history.truncate(5);
    }
}
