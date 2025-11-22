use crate::models::{CityMap, GameMode, VictoryResult};
use issun::prelude::*;

/// Pure win condition checking service (stateless)
#[derive(Clone, Default, DeriveService)]
#[service(name = "win_condition_service")]
pub struct WinConditionService;

impl WinConditionService {
    /// Check victory conditions for Plague mode
    pub fn check_plague_victory(
        &self,
        city_map: &CityMap,
        turn: u32,
        max_turns: u32,
    ) -> Option<VictoryResult> {
        let (total_pop, total_infected) = self.calculate_totals(city_map);
        let infection_rate = if total_pop > 0 {
            total_infected as f32 / total_pop as f32
        } else {
            0.0
        };

        if infection_rate >= 0.7 {
            Some(VictoryResult::Victory(format!(
                "Infection rate: {:.1}%",
                infection_rate * 100.0
            )))
        } else if turn >= max_turns {
            Some(VictoryResult::Defeat(format!(
                "Failed. Infection only reached: {:.1}%",
                infection_rate * 100.0
            )))
        } else {
            None
        }
    }

    /// Check victory conditions for Savior mode
    pub fn check_savior_victory(
        &self,
        city_map: &CityMap,
        turn: u32,
        max_turns: u32,
    ) -> Option<VictoryResult> {
        let (total_pop, total_infected) = self.calculate_totals(city_map);
        let infection_rate = if total_pop > 0 {
            total_infected as f32 / total_pop as f32
        } else {
            1.0
        };
        let survival_rate = 1.0 - infection_rate;

        if infection_rate >= 0.7 {
            Some(VictoryResult::Defeat(format!(
                "City overwhelmed. Infection: {:.1}%",
                infection_rate * 100.0
            )))
        } else if turn >= max_turns {
            if survival_rate >= 0.6 {
                Some(VictoryResult::Victory(format!(
                    "City saved! Survival rate: {:.1}%",
                    survival_rate * 100.0
                )))
            } else {
                Some(VictoryResult::Defeat(format!(
                    "Not enough survivors. Survival rate: {:.1}%",
                    survival_rate * 100.0
                )))
            }
        } else {
            None
        }
    }

    /// Helper to calculate totals
    fn calculate_totals(&self, city_map: &CityMap) -> (u32, u32) {
        let total_pop = city_map.districts.iter().map(|d| d.population).sum::<u32>();
        let total_infected = city_map.districts.iter().map(|d| d.infected).sum::<u32>();
        (total_pop, total_infected)
    }
}
