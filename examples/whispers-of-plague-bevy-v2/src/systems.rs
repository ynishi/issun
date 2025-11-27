//! Game systems - using issun-core contagion mechanics

use crate::components::*;
use crate::resources::{GameContext, GameMode, UIState, VictoryResult, VictoryState};
use crate::states::GameScene;
use bevy::{ecs::message::MessageReader, prelude::*};
use issun_bevy::plugins::contagion_v2::*;

/// Initialize first infected district
pub fn setup_initial_infection(mut _commands: Commands) {
    // Initial infection will be set by infect_initial_district system
}

/// Infect the initial district when game starts
pub fn infect_initial_district(
    mut query: Query<(
        &District,
        &mut ContagionState<PlagueVirus>,
        &mut ContagionInputParams,
    )>,
    mut ui_state: ResMut<UIState>,
) {
    // Infect Downtown district
    for (district, mut state, mut params) in query.iter_mut() {
        if district.id == "downtown" {
            state.state.severity = 100; // Start with severity 100
            params.density = 0.8; // High initial infection density
            ui_state.add_message(format!("Patient Zero detected in {}!", district.name));
            break;
        }
    }
}

/// Propagate infection between districts based on ContagionGraph edges
pub fn propagate_infection_between_districts_system(
    mut query: Query<(
        &District,
        &mut ContagionState<PlagueVirus>,
        &mut ContagionInputParams,
    )>,
    contagion_graph: Res<crate::resources::ContagionGraph>,
    game_context: Res<GameContext>,
    mut ui_state: ResMut<UIState>,
) {
    // Only update on turn change
    if !game_context.is_changed() {
        return;
    }

    // Collect current infection states to avoid borrow issues
    let district_states: std::collections::HashMap<String, (u32, String)> = query
        .iter()
        .map(|(district, state, _)| {
            (
                district.id.clone(),
                (state.severity(), district.name.clone()),
            )
        })
        .collect();

    // Update density and directly infect districts based on infection pressure
    for (district, mut state, mut params) in query.iter_mut() {
        // Calculate infection pressure from neighbors
        let mut infection_pressure = 0.0;

        for edge in &contagion_graph.edges {
            if edge.to == district.id {
                if let Some(&(source_severity, _)) = district_states.get(&edge.from) {
                    if source_severity > 0 {
                        // Infection pressure = edge rate * source severity
                        infection_pressure += edge.rate * (source_severity as f32 / 100.0);
                    }
                }
            }
        }

        let current_severity = district_states
            .get(&district.id)
            .map(|(s, _)| *s)
            .unwrap_or(0);

        // Calculate effective density = infection_pressure + panic_level bonus
        let panic_bonus = district.panic_level; // panic_level ranges 0.0 to 1.0
        let effective_density = (infection_pressure + panic_bonus).min(1.0);

        // If not infected yet and infection pressure is high enough, start infection
        if current_severity == 0 && infection_pressure > 0.15 {
            // Start infection with severity based on pressure
            let initial_severity = (infection_pressure * 50.0).min(20.0) as u32;
            state.state.severity = initial_severity;
            params.density = effective_density; // Use effective density

            ui_state.add_message(format!(
                "{} INFECTED! Initial severity: {} (pressure: {:.2}, panic: {:.1}%)",
                district.name,
                initial_severity,
                infection_pressure,
                panic_bonus * 100.0
            ));
        } else if current_severity == 0 && infection_pressure > 0.0 {
            // Not yet infected but exposed - update density
            params.density = effective_density;
        } else if current_severity > 0 {
            // Already infected - use effective density (base + panic)
            params.density = (0.5 + panic_bonus).min(1.0);
        }
    }
}

/// Sync ContagionState severity to District infected count
pub fn sync_contagion_to_district_system(
    mut query: Query<(&mut District, &ContagionState<PlagueVirus>)>,
    game_context: Res<GameContext>,
) {
    // Only update on turn change
    if !game_context.is_changed() {
        return;
    }

    for (mut district, state) in query.iter_mut() {
        // Convert severity to infected population (each severity point = 10 infected)
        let new_infected = (state.severity() * 10).min(district.population);

        // Calculate deaths (5% lethality per turn)
        let deaths = (district.infected as f32 * 0.05).round() as u32;
        if deaths > 0 {
            district.dead += deaths;
            district.infected = district.infected.saturating_sub(deaths);
        }

        // Update infected count
        district.infected = new_infected;
    }
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
    let total_dead: u32 = districts.iter().map(|d| d.dead).sum();

    let infection_rate = if total_pop > 0 {
        total_infected as f32 / total_pop as f32
    } else {
        0.0
    };

    let death_rate = if total_pop > 0 {
        total_dead as f32 / total_pop as f32
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
                    "Plague Defeat. Final infection rate: {:.1}%",
                    infection_rate * 100.0
                )))
            } else {
                None
            }
        }
        GameMode::Savior => {
            if death_rate >= 0.3 {
                Some(VictoryState::Defeat(format!(
                    "Savior Defeat. Death toll: {:.1}%",
                    death_rate * 100.0
                )))
            } else if game_context.turn >= game_context.max_turns {
                Some(VictoryState::Victory(format!(
                    "Savior Victory! Kept deaths under 30%. Final: {:.1}%",
                    death_rate * 100.0
                )))
            } else {
                None
            }
        }
    };

    if let Some(state) = result {
        victory_result.0 = Some(state);
        next_state.set(GameScene::Result);
    }
}

// Log contagion events - commented out until we expose ContagionEventWrapper
// pub fn log_contagion_messages(
//     mut messages: MessageReader<ContagionEventWrapper>,
//     districts: Query<&District>,
//     mut ui_state: ResMut<UIState>,
// ) {
//     for wrapper in messages.read() {
//         // Find which district this entity belongs to
//         if let Ok(district) = districts.get(wrapper.entity) {
//             match wrapper.event {
//                 issun_core::mechanics::contagion::ContagionEvent::Infected => {
//                     ui_state.add_message(format!("{} infection started!", district.name));
//                 }
//                 issun_core::mechanics::contagion::ContagionEvent::Progressed { new_severity } => {
//                     ui_state.add_message(format!(
//                         "{} infection progressed to severity {}",
//                         district.name, new_severity
//                     ));
//                 }
//             }
//         }
//     }
// }
