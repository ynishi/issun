use crate::models::{CityMap, GameMode, PlagueGameContext, VictoryResult, Virus};
use crate::services::{VirusService, WinConditionService};
use issun::prelude::*;

/// Turn management system (stateful orchestration)
#[derive(DeriveSystem)]
#[system(name = "turn_system")]
pub struct TurnSystem {
    virus_service: VirusService,
    win_condition_service: WinConditionService,
}

impl TurnSystem {
    pub fn new() -> Self {
        Self {
            virus_service: VirusService,
            win_condition_service: WinConditionService,
        }
    }

    /// Seed initial infection in first district
    pub async fn seed_infection(&mut self, resources: &mut ResourceContext) {
        if let Some(mut city_map) = resources.get_mut::<CityMap>().await {
            if let Some(district) = city_map.districts.first_mut() {
                district.infected = 100; // Increased from 10 for more visible initial spread
            }
        }
    }

    /// Execute next turn: spread virus, check mutation, return logs
    /// Note: Rumor decay should be called separately by the scene handler
    pub async fn next_turn(&mut self, resources: &mut ResourceContext) -> Vec<String> {
        let mut logs = vec![];

        // 1. Increment turn
        {
            if let Some(mut ctx) = resources.get_mut::<PlagueGameContext>().await {
                ctx.turn += 1;
                logs.push(format!("=== Turn {} ===", ctx.turn));
            }
        }

        // 2. Read virus (clone to avoid holding borrow)
        let virus = match resources.get::<Virus>().await {
            Some(v) => v.clone(),
            None => {
                logs.push("ERROR: Virus not found".into());
                return logs;
            }
        };

        // 3. Spread virus in all districts
        {
            if let Some(mut city_map) = resources.get_mut::<CityMap>().await {
                // First pass: within-district spread
                let mut district_infections = vec![];
                for district in &mut city_map.districts {
                    let new_infected =
                        self.virus_service
                            .calculate_spread(district, &virus, district.panic_level);
                    district.infected = district.infected.saturating_add(new_infected);

                    let new_deaths = self
                        .virus_service
                        .calculate_deaths(district.infected, &virus);
                    district.dead = district.dead.saturating_add(new_deaths);

                    district_infections.push((
                        district.name.clone(),
                        new_infected,
                        new_deaths,
                        district.infected,
                        district.dead,
                    ));
                }

                // Second pass: inter-district spread (simple model: spread to adjacent districts)
                let district_count = city_map.districts.len();
                for i in 0..district_count {
                    if city_map.districts[i].infected > 0 {
                        // Calculate spillover to neighboring districts
                        let spillover =
                            (city_map.districts[i].infected as f32 * 0.05 * virus.spread_rate)
                                .round() as u32;

                        if spillover > 0 {
                            // Spread to next district (circular)
                            let next_idx = (i + 1) % district_count;
                            if city_map.districts[next_idx].healthy() > 0 {
                                let spread_amount =
                                    spillover.min(city_map.districts[next_idx].healthy());
                                city_map.districts[next_idx].infected = city_map.districts
                                    [next_idx]
                                    .infected
                                    .saturating_add(spread_amount);
                            }
                        }
                    }
                }

                // Log results
                for (name, new_inf, new_deaths, total_inf, total_dead) in district_infections {
                    logs.push(format!(
                        "{}: +{} infected, +{} deaths (Total: {}/{})",
                        name, new_inf, new_deaths, total_inf, total_dead
                    ));
                }
            }
        }

        // 4. Check for mutation
        let total_infected = {
            resources
                .get::<CityMap>()
                .await
                .map(|city| city.districts.iter().map(|d| d.infected).sum::<u32>())
                .unwrap_or(0)
        };

        if self
            .virus_service
            .should_mutate(total_infected, virus.mutation_stage)
        {
            let new_virus = self.virus_service.mutate(&virus);
            logs.push(format!("🦠 Virus mutated to {}!", new_virus.name));
            resources.insert(new_virus);
        }

        logs
    }

    /// Check if victory or defeat conditions are met
    pub async fn check_victory(&self, resources: &ResourceContext) -> Option<VictoryResult> {
        let ctx = resources.get::<PlagueGameContext>().await?;
        let city_map = resources.get::<CityMap>().await?;

        match ctx.mode {
            GameMode::Plague => {
                self.win_condition_service
                    .check_plague_victory(&city_map, ctx.turn, ctx.max_turns)
            }
            GameMode::Savior => {
                self.win_condition_service
                    .check_savior_victory(&city_map, ctx.turn, ctx.max_turns)
            }
        }
    }
}

impl Default for TurnSystem {
    fn default() -> Self {
        Self::new()
    }
}
