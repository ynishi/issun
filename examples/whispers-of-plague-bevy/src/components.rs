use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// District entity component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct District {
    pub id: String,
    pub name: String,
    pub population: u32,
    pub infected: u32,
    pub dead: u32,
    pub panic_level: f32,
}

impl District {
    pub fn new(id: impl Into<String>, name: impl Into<String>, population: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            population,
            infected: 0,
            dead: 0,
            panic_level: 0.2,
        }
    }

    pub fn healthy(&self) -> u32 {
        self.population.saturating_sub(self.infected + self.dead)
    }

    pub fn infection_rate(&self) -> f32 {
        if self.population == 0 {
            0.0
        } else {
            self.infected as f32 / self.population as f32
        }
    }
}

/// Contagion entity component
#[derive(Component, Debug, Clone)]
pub struct Contagion {
    pub id: String,
    pub location: String,
    pub turns_alive: u32,
    pub severity: DiseaseLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiseaseLevel {
    Mild,
    Moderate,
    Severe,
}

impl DiseaseLevel {
    pub fn spread_rate(&self) -> f32 {
        match self {
            DiseaseLevel::Mild => 0.35,
            DiseaseLevel::Moderate => 0.45, // +30%
            DiseaseLevel::Severe => 0.53,   // +50%
        }
    }

    pub fn lethality(&self) -> f32 {
        match self {
            DiseaseLevel::Mild => 0.05,
            DiseaseLevel::Moderate => 0.06, // +20%
            DiseaseLevel::Severe => 0.07,   // +40%
        }
    }
}
