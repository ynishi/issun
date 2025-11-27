//! Components for Whispers of Plague V2 - using issun-core

use bevy::prelude::*;
use issun_core::mechanics::contagion::presets::*;
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

/// Type alias for plague virus using issun-core
pub type PlagueVirus = ZombieVirus;

/// Marker component for districts with contagion
#[derive(Component)]
pub struct HasContagion;
