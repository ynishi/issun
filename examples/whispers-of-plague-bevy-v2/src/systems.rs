//! Game systems - using issun-core contagion mechanics

use crate::components::*;
use crate::resources::{GameContext, GameMode, UIState, VictoryResult, VictoryState};
use crate::states::GameScene;
use bevy::prelude::*;
use issun_bevy::plugins::contagion_v2::*;
use issun_core::mechanics::propagation::*;
use issun_core::mechanics::{EventEmitter, Mechanic};

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

/// Propagate infection between districts using PropagationMechanic
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

    // Step 1: Convert ContagionGraph to PropagationGraph
    let prop_graph = PropagationGraph::new(
        contagion_graph
            .edges
            .iter()
            .map(|e| PropagationEdge::new(e.from.clone(), e.to.clone(), e.rate))
            .collect(),
    );

    // Step 2: Collect current infection states
    let district_data: std::collections::HashMap<String, (u32, String, f32)> = query
        .iter()
        .map(|(district, state, _)| {
            (
                district.id.clone(),
                (state.severity(), district.name.clone(), district.panic_level),
            )
        })
        .collect();

    // Step 3: Create PropagationInput
    let mut node_states = std::collections::HashMap::new();
    for (id, (severity, _, _)) in &district_data {
        node_states.insert(id.clone(), *severity as f32);
    }
    let input = PropagationInput { node_states };

    // Step 4: Run PropagationMechanic
    let mut prop_state = PropagationState::default();
    let mut events = Vec::new();
    let mut emitter = VecEmitter { events: &mut events };

    LinearPropagationMechanic::step(&prop_graph, &mut prop_state, input, &mut emitter);

    // Step 5: Apply propagation results to districts
    for (district, mut state, mut params) in query.iter_mut() {
        let current_severity = district_data
            .get(&district.id)
            .map(|(s, _, _)| *s)
            .unwrap_or(0);

        let panic_level = district.panic_level;
        let infection_pressure = prop_state.get_pressure(&district.id);

        // Calculate effective density = infection_pressure + panic_level bonus
        let effective_density = (infection_pressure + panic_level).min(1.0);

        // Apply infection events
        for event in &events {
            match event {
                PropagationEvent::InitialInfection {
                    node,
                    initial_severity,
                } if node == &district.id => {
                    state.state.severity = *initial_severity;
                    params.density = effective_density;

                    ui_state.add_message(format!(
                        "{} INFECTED! Initial severity: {} (pressure: {:.2}, panic: {:.1}%)",
                        district.name,
                        initial_severity,
                        infection_pressure,
                        panic_level * 100.0
                    ));
                }
                _ => {}
            }
        }

        // Update density based on infection status
        if current_severity == 0 && infection_pressure > 0.0 {
            // Not yet infected but exposed - update density
            params.density = effective_density;
        } else if current_severity > 0 {
            // Already infected - use effective density (base + panic)
            params.density = (0.5 + panic_level).min(1.0);
        }
    }
}

/// Event emitter that collects events into a Vec
struct VecEmitter<'a, E> {
    events: &'a mut Vec<E>,
}

impl<'a, E> EventEmitter<E> for VecEmitter<'a, E> {
    fn emit(&mut self, event: E) {
        self.events.push(event);
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
