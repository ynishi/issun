//! Game rules and win/lose conditions

use bevy::prelude::*;
use issun_bevy::plugins::contagion::*;

use crate::player::*;
use crate::world::*;

/// Game state
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    #[default]
    Playing,
    Victory(VictoryType),
    Defeat(DefeatType),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum VictoryType {
    CureDeployed,
    NaturalContainment,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DefeatType {
    GlobalPandemic,
    CriticalMutations,
    EconomicCollapse,
}

/// Game statistics
#[derive(Resource, Default)]
pub struct GameStats {
    pub current_turn: u32,
    pub total_infected: usize,
    pub total_active: usize,
    pub total_recovered: usize,
    pub total_deaths: usize,
    pub low_infection_streak: u32,
}

impl GameStats {
    pub fn infection_rate(&self) -> f32 {
        let total_pop = get_total_population();
        if total_pop > 0 {
            self.total_infected as f32 / total_pop as f32
        } else {
            0.0
        }
    }

    pub fn active_rate(&self) -> f32 {
        let total_pop = get_total_population();
        if total_pop > 0 {
            self.total_active as f32 / total_pop as f32
        } else {
            0.0
        }
    }
}

/// Update game statistics
pub fn update_game_stats(
    mut stats: ResMut<GameStats>,
    infections: Query<&ContagionInfection>,
) {
    let mut total_infected = 0;
    let mut total_active = 0;
    let mut total_recovered = 0;

    for infection in infections.iter() {
        total_infected += 1;
        match infection.state {
            InfectionState::Active { .. } => total_active += 1,
            InfectionState::Recovered { .. } => total_recovered += 1,
            _ => {}
        }
    }

    stats.total_infected = total_infected;
    stats.total_active = total_active;
    stats.total_recovered = total_recovered;

    // Update low infection streak
    if stats.infection_rate() < 0.1 {
        stats.low_infection_streak += 1;
    } else {
        stats.low_infection_streak = 0;
    }
}

/// Check victory conditions
pub fn check_victory_conditions(
    stats: Res<GameStats>,
    cure_research: Res<CureResearch>,
    mut game_state: ResMut<GameState>,
) {
    if *game_state != GameState::Playing {
        return;
    }

    // Victory: Cure deployed and complete
    if cure_research.deployed && cure_research.deployment_complete(stats.current_turn) {
        info!("VICTORY: Cure deployed successfully!");
        *game_state = GameState::Victory(VictoryType::CureDeployed);
        return;
    }

    // Victory: Natural containment
    if stats.low_infection_streak >= 15 {
        info!("VICTORY: Natural containment achieved!");
        *game_state = GameState::Victory(VictoryType::NaturalContainment);
        return;
    }
}

/// Check defeat conditions
pub fn check_defeat_conditions(
    stats: Res<GameStats>,
    contagions: Query<&Contagion>,
    quarantines: Res<ActiveQuarantines>,
    mut game_state: ResMut<GameState>,
) {
    if *game_state != GameState::Playing {
        return;
    }

    // Defeat: Global pandemic (70%+ infected)
    if stats.infection_rate() >= 0.7 {
        warn!("DEFEAT: Global pandemic - 70%+ infected!");
        *game_state = GameState::Defeat(DefeatType::GlobalPandemic);
        return;
    }

    // Defeat: Critical mutations (5+ critical severity)
    let critical_count = contagions
        .iter()
        .filter(|c| {
            matches!(
                c.content,
                ContagionContent::Disease {
                    severity: DiseaseLevel::Critical,
                    ..
                }
            )
        })
        .count();

    if critical_count >= 5 {
        warn!("DEFEAT: {} critical mutations overwhelm response!", critical_count);
        *game_state = GameState::Defeat(DefeatType::CriticalMutations);
        return;
    }

    // Defeat: Economic collapse (50%+ cities quarantined for 10+ turns)
    let total_cities = CITIES.len();
    let quarantined_count = quarantines.active_count(stats.current_turn);
    let quarantine_rate = quarantined_count as f32 / total_cities as f32;

    if quarantine_rate >= 0.5 {
        // Check if this has been sustained for 10 turns
        // (Simplified: just check current state for now)
        warn!("DEFEAT: Economic collapse - too many quarantines!");
        *game_state = GameState::Defeat(DefeatType::EconomicCollapse);
        return;
    }
}

/// Advance turn
pub fn advance_turn(mut stats: ResMut<GameStats>) {
    stats.current_turn += 1;
    info!("=== Turn {} ===", stats.current_turn);
}
