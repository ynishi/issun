//! Plant types and properties for garden simulation

use serde::{Deserialize, Serialize};

/// Plant species with different growth characteristics
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlantSpecies {
    Tomato,
    Lettuce,
    Carrot,
    Wheat,
    Sunflower,
}

impl PlantSpecies {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            PlantSpecies::Tomato => "Tomato",
            PlantSpecies::Lettuce => "Lettuce",
            PlantSpecies::Carrot => "Carrot",
            PlantSpecies::Wheat => "Wheat",
            PlantSpecies::Sunflower => "Sunflower",
        }
    }

    /// Get plant icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            PlantSpecies::Tomato => "🍅",
            PlantSpecies::Lettuce => "🥬",
            PlantSpecies::Carrot => "🥕",
            PlantSpecies::Wheat => "🌾",
            PlantSpecies::Sunflower => "🌻",
        }
    }

    /// Get growth rate (progress per tick)
    pub fn growth_rate(&self) -> f32 {
        match self {
            PlantSpecies::Tomato => 2.0,
            PlantSpecies::Lettuce => 5.0, // Fast growing
            PlantSpecies::Carrot => 3.0,
            PlantSpecies::Wheat => 4.0,
            PlantSpecies::Sunflower => 1.5, // Slow but valuable
        }
    }

    /// Get decay rate (durability loss per tick)
    pub fn decay_rate(&self) -> f32 {
        match self {
            PlantSpecies::Tomato => 0.3,
            PlantSpecies::Lettuce => 0.5, // Fragile
            PlantSpecies::Carrot => 0.2,  // Hardy
            PlantSpecies::Wheat => 0.3,
            PlantSpecies::Sunflower => 0.1, // Very hardy
        }
    }

    /// Get max growth value
    pub fn max_growth(&self) -> f32 {
        100.0
    }

    /// Get max durability (health)
    pub fn max_durability(&self) -> f32 {
        match self {
            PlantSpecies::Tomato => 100.0,
            PlantSpecies::Lettuce => 80.0, // Fragile
            PlantSpecies::Carrot => 120.0, // Hardy
            PlantSpecies::Wheat => 100.0,
            PlantSpecies::Sunflower => 150.0, // Very hardy
        }
    }
}

/// Plant growth stage based on generation progress
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrowthStage {
    Seed,     // 0-20%
    Seedling, // 20-60%
    Growing,  // 60-90%
    Mature,   // 90-100%
    Ready,    // 100% - ready to harvest
}

impl GrowthStage {
    pub fn from_progress(progress_ratio: f32) -> Self {
        if progress_ratio >= 1.0 {
            GrowthStage::Ready
        } else if progress_ratio >= 0.9 {
            GrowthStage::Mature
        } else if progress_ratio >= 0.6 {
            GrowthStage::Growing
        } else if progress_ratio >= 0.2 {
            GrowthStage::Seedling
        } else {
            GrowthStage::Seed
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            GrowthStage::Seed => "○",
            GrowthStage::Seedling => "◐",
            GrowthStage::Growing => "◑",
            GrowthStage::Mature => "◉",
            GrowthStage::Ready => "★",
        }
    }
}

/// Plant health based on durability
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlantHealth {
    Healthy,  // 80-100%
    Good,     // 50-80%
    Stressed, // 20-50%
    Dying,    // 0-20%
    Dead,     // 0%
}

impl PlantHealth {
    pub fn from_durability_ratio(ratio: f32) -> Self {
        if ratio <= 0.0 {
            PlantHealth::Dead
        } else if ratio < 0.2 {
            PlantHealth::Dying
        } else if ratio < 0.5 {
            PlantHealth::Stressed
        } else if ratio < 0.8 {
            PlantHealth::Good
        } else {
            PlantHealth::Healthy
        }
    }
}

/// Garden plot position
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct PlotPosition {
    pub x: usize,
    pub y: usize,
}

#[allow(dead_code)]
impl PlotPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
