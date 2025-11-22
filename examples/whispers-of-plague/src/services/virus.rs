use crate::models::{District, Virus};
use issun::prelude::*;

/// Pure virus calculation service (stateless)
#[derive(Clone, Default, DeriveService)]
#[service(name = "virus_service")]
pub struct VirusService;

impl VirusService {
    /// Calculate new infections for a district
    pub fn calculate_spread(&self, district: &District, virus: &Virus, panic: f32) -> u32 {
        let healthy = district.healthy();
        if healthy == 0 || district.infected == 0 {
            return 0;
        }

        let base_spread =
            (district.infected as f32 * virus.spread_rate * (1.0 + panic)).round() as u32;
        base_spread.min(healthy)
    }

    /// Calculate deaths from current infections
    pub fn calculate_deaths(&self, infected: u32, virus: &Virus) -> u32 {
        ((infected as f32) * virus.lethality).round() as u32
    }

    /// Check if virus should mutate based on infection count and stage
    pub fn should_mutate(&self, total_infected: u32, current_stage: u32) -> bool {
        let threshold = match current_stage {
            0 => 50000,
            1 => 100000,
            _ => u32::MAX,
        };
        total_infected >= threshold
    }

    /// Generate mutated virus
    pub fn mutate(&self, virus: &Virus) -> Virus {
        let new_stage = virus.mutation_stage + 1;
        match new_stage {
            1 => Virus {
                name: "Beta Strain".into(),
                spread_rate: virus.spread_rate * 1.3,
                lethality: virus.lethality * 1.2,
                mutation_stage: 1,
            },
            2 => Virus {
                name: "Gamma Strain".into(),
                spread_rate: virus.spread_rate * 1.5,
                lethality: virus.lethality * 1.4,
                mutation_stage: 2,
            },
            _ => virus.clone(),
        }
    }
}
