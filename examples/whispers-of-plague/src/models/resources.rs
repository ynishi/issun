use serde::{Deserialize, Serialize};

/// Game mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Plague,
    Savior,
}

/// Victory/defeat result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VictoryResult {
    Victory(String),
    Defeat(String),
}

/// City map containing all districts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityMap {
    pub districts: Vec<District>,
}

impl CityMap {
    pub fn new() -> Self {
        Self {
            districts: vec![
                District::new("downtown", "Downtown", 10000),
                District::new("industrial", "Industrial Zone", 8000),
                District::new("residential", "Residential Area", 15000),
                District::new("suburbs", "Suburbs", 12000),
                District::new("harbor", "Harbor District", 9000),
            ],
        }
    }
}

impl Default for CityMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual district in the city
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Virus strain with mutation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Virus {
    pub name: String,
    pub spread_rate: f32,
    pub lethality: f32,
    pub mutation_stage: u32,
}

impl Virus {
    pub fn initial() -> Self {
        Self {
            name: "Alpha Strain".into(),
            spread_rate: 0.35, // Increased from 0.15 for faster spread
            lethality: 0.05,
            mutation_stage: 0,
        }
    }
}
