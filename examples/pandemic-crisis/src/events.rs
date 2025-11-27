//! Event handling and logging

use bevy::prelude::*;
use issun_bevy::plugins::contagion::*;

use crate::world::get_city_name;

/// Event log for display
#[derive(Resource, Default)]
pub struct EventLog {
    pub entries: Vec<String>,
    pub max_entries: usize,
}

impl EventLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add(&mut self, message: String) {
        self.entries.push(message);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn recent(&self, count: usize) -> &[String] {
        let start = self.entries.len().saturating_sub(count);
        &self.entries[start..]
    }
}

/// Handle contagion spawned events
pub fn handle_contagion_spawned(
    mut event_reader: MessageReader<ContagionSpawnedEvent>,
    nodes: Query<&ContagionNode>,
    mut event_log: ResMut<EventLog>,
) {
    for event in event_reader.read() {
        if let Ok(node) = nodes.get(event.origin_node) {
            let city_name = get_city_name(&node.node_id);
            event_log.add(format!("⚠️  Virus X detected in {}", city_name));
            info!("Virus X spawned in {}", city_name);
        }
    }
}

/// Handle contagion spread events
pub fn handle_contagion_spread(
    mut event_reader: MessageReader<ContagionSpreadEvent>,
    infections: Query<&ContagionInfection>,
    nodes: Query<&ContagionNode>,
    mut event_log: ResMut<EventLog>,
) {
    for event in event_reader.read() {
        if let Ok(infection) = infections.get(event.infection_entity) {
            if let Ok(node) = nodes.get(infection.node_entity) {
                let city_name = get_city_name(&node.node_id);

                if event.is_mutation {
                    event_log.add(format!("🧬 Mutation detected spreading to {}", city_name));
                    warn!("Mutation spread to {}", city_name);
                } else {
                    event_log.add(format!("🦠 Disease spread to {}", city_name));
                    info!("Disease spread to {}", city_name);
                }
            }
        }
    }
}

/// Handle infection state changes
pub fn handle_state_changes(
    mut event_reader: MessageReader<InfectionStateChangedEvent>,
    nodes: Query<&ContagionNode>,
    mut event_log: ResMut<EventLog>,
) {
    for event in event_reader.read() {
        if let Ok(node) = nodes.get(event.node_entity) {
            let city_name = get_city_name(&node.node_id);

            match (event.old_state, event.new_state) {
                (InfectionStateType::Incubating, InfectionStateType::Active) => {
                    event_log.add(format!("📈 {} - Incubation period ended, active outbreak!", city_name));
                    warn!("{} - Active outbreak!", city_name);
                }
                (InfectionStateType::Active, InfectionStateType::Recovered) => {
                    event_log.add(format!("📉 {} - Infections recovering", city_name));
                }
                (InfectionStateType::Recovered, InfectionStateType::Plain) => {
                    event_log.add(format!("✅ {} - Immunity period ended", city_name));
                }
                _ => {}
            }
        }
    }
}

/// Handle propagation completion
pub fn handle_propagation_complete(
    mut event_reader: MessageReader<PropagationStepCompletedEvent>,
    mut event_log: ResMut<EventLog>,
) {
    for event in event_reader.read() {
        if event.spread_count > 0 {
            event_log.add(format!(
                "🌍 Propagation: {} new infections, {} mutations",
                event.spread_count, event.mutation_count
            ));
        }
    }
}
