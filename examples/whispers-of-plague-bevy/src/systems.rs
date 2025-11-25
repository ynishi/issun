use crate::components::{Contagion, DiseaseLevel, District};
use crate::resources::{ContagionGraph, GameContext, GameMode, UIState, VictoryResult, VictoryState};
use crate::states::GameScene;
use bevy::prelude::*;

/// Process turn - spread contagion through graph
pub fn spread_contagion_system(
    mut districts: Query<&mut District>,
    contagions: Query<&Contagion>,
    contagion_graph: Res<ContagionGraph>,
    game_context: Res<GameContext>,
    mut ui_state: ResMut<UIState>,
) {
    // Only process on turn change
    if !game_context.is_changed() {
        return;
    }

    // Get current disease severity
    let severity = contagions
        .iter()
        .next()
        .map(|c| c.severity)
        .unwrap_or(DiseaseLevel::Mild);

    let spread_rate = severity.spread_rate();
    let lethality = severity.lethality();

    // Collect district states to avoid borrow conflicts
    let district_states: Vec<(String, u32, f32)> = districts
        .iter()
        .map(|d| (d.id.clone(), d.infected, d.panic_level))
        .collect();

    // Spread infection through edges
    for edge in &contagion_graph.edges {
        if let Some((_, from_infected, panic)) = district_states
            .iter()
            .find(|(id, _, _)| id == &edge.from)
        {
            let spread_count = calculate_spread(*from_infected, edge.rate, spread_rate, *panic);

            if spread_count > 0 {
                // Find target district and update
                for mut district in districts.iter_mut() {
                    if district.id == edge.to {
                        let healthy = district.healthy();
                        let new_infections = spread_count.min(healthy);
                        district.infected += new_infections;

                        if new_infections > 0 {
                            ui_state.add_message(format!(
                                "{} -> {}: {} new infections",
                                edge.from, edge.to, new_infections
                            ));
                        }
                        break;
                    }
                }
            }
        }
    }

    // Apply lethality
    for mut district in districts.iter_mut() {
        let deaths = (district.infected as f32 * lethality).round() as u32;
        if deaths > 0 {
            district.dead += deaths;
            district.infected = district.infected.saturating_sub(deaths);
        }
    }
}

/// Calculate infection spread
fn calculate_spread(infected: u32, edge_rate: f32, spread_rate: f32, panic: f32) -> u32 {
    let base_spread = (infected as f32 * edge_rate * spread_rate * (1.0 + panic))
        .round() as u32;
    base_spread
}

/// Check win/loss conditions
pub fn check_win_condition_system(
    districts: Query<&District>,
    game_context: Res<GameContext>,
    mut victory_result: ResMut<VictoryResult>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let total_pop: u32 = districts.iter().map(|d| d.population).sum();
    let total_infected: u32 = districts.iter().map(|d| d.infected).sum();
    let infection_rate = if total_pop > 0 {
        total_infected as f32 / total_pop as f32
    } else {
        0.0
    };

    let result = match game_context.mode {
        GameMode::Plague => {
            if infection_rate >= 0.7 {
                Some(VictoryState::Victory(format!(
                    "Plague Victory! Infection rate: {:.1}%",
                    infection_rate * 100.0
                )))
            } else if game_context.turn >= game_context.max_turns {
                Some(VictoryState::Defeat(format!(
                    "Plague Defeated. Infection only reached: {:.1}%",
                    infection_rate * 100.0
                )))
            } else {
                None
            }
        }
        GameMode::Savior => {
            if infection_rate >= 0.7 {
                Some(VictoryState::Defeat(format!(
                    "City Overwhelmed. Infection: {:.1}%",
                    infection_rate * 100.0
                )))
            } else if game_context.turn >= game_context.max_turns {
                let survival_rate = 1.0 - infection_rate;
                if survival_rate >= 0.6 {
                    Some(VictoryState::Victory(format!(
                        "City Saved! Survival rate: {:.1}%",
                        survival_rate * 100.0
                    )))
                } else {
                    Some(VictoryState::Defeat(format!(
                        "Not Enough Survivors. Survival rate: {:.1}%",
                        survival_rate * 100.0
                    )))
                }
            } else {
                None
            }
        }
    };

    if let Some(r) = result {
        victory_result.0 = Some(r);
        next_state.set(GameScene::Result);
    }
}

/// Setup initial infection
pub fn setup_initial_infection(mut commands: Commands) {
    commands.spawn(Contagion {
        id: "alpha_virus".to_string(),
        location: "downtown".to_string(),
        turns_alive: 0,
        severity: DiseaseLevel::Mild,
    });
}

/// Infect initial district
pub fn infect_initial_district(mut districts: Query<&mut District>) {
    for mut district in districts.iter_mut() {
        if district.id == "downtown" {
            district.infected = 100;
            break;
        }
    }
}

/// Mutate virus based on infection count
pub fn mutate_virus_system(
    districts: Query<&District>,
    mut contagions: Query<&mut Contagion>,
    mut ui_state: ResMut<UIState>,
) {
    let total_infected: u32 = districts.iter().map(|d| d.infected).sum();

    for mut contagion in contagions.iter_mut() {
        let new_severity = match contagion.severity {
            DiseaseLevel::Mild if total_infected >= 5000 => {
                ui_state.add_message("⚠ Virus mutated to Beta strain!".to_string());
                Some(DiseaseLevel::Moderate)
            }
            DiseaseLevel::Moderate if total_infected >= 10000 => {
                ui_state.add_message("⚠⚠ Virus mutated to Gamma strain!".to_string());
                Some(DiseaseLevel::Severe)
            }
            _ => None,
        };

        if let Some(severity) = new_severity {
            contagion.severity = severity;
        }
    }
}
