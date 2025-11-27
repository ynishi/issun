//! Disease mechanics and configuration

use bevy::prelude::*;
use issun_bevy::plugins::contagion::*;
use rand::Rng;

use crate::world::CITIES;

/// Difficulty level
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn to_config(self) -> ContagionConfig {
        match self {
            Difficulty::Easy => ContagionConfig {
                global_propagation_rate: 0.5,
                default_mutation_rate: 0.1,
                lifetime_turns: 25,
                min_credibility: 0.1,
                time_mode: TimeMode::TurnBased,
                incubation_transmission_rate: 0.15,
                active_transmission_rate: 0.6,
                recovered_transmission_rate: 0.05,
                plain_transmission_rate: 0.0,
                default_incubation_duration: DurationConfig::new(5.0, 0.2),
                default_active_duration: DurationConfig::new(7.0, 0.2),
                default_immunity_duration: DurationConfig::new(15.0, 0.3),
                default_reinfection_enabled: true,
            },
            Difficulty::Normal => ContagionConfig {
                global_propagation_rate: 0.6,
                default_mutation_rate: 0.15,
                lifetime_turns: 20,
                min_credibility: 0.1,
                time_mode: TimeMode::TurnBased,
                incubation_transmission_rate: 0.2,
                active_transmission_rate: 0.8,
                recovered_transmission_rate: 0.05,
                plain_transmission_rate: 0.0,
                default_incubation_duration: DurationConfig::new(3.0, 0.3),
                default_active_duration: DurationConfig::new(5.0, 0.2),
                default_immunity_duration: DurationConfig::new(10.0, 0.5),
                default_reinfection_enabled: true,
            },
            Difficulty::Hard => ContagionConfig {
                global_propagation_rate: 0.7,
                default_mutation_rate: 0.25,
                lifetime_turns: 15,
                min_credibility: 0.1,
                time_mode: TimeMode::TurnBased,
                incubation_transmission_rate: 0.3,
                active_transmission_rate: 0.9,
                recovered_transmission_rate: 0.1,
                plain_transmission_rate: 0.0,
                default_incubation_duration: DurationConfig::new(2.0, 0.3),
                default_active_duration: DurationConfig::new(4.0, 0.2),
                default_immunity_duration: DurationConfig::new(8.0, 0.5),
                default_reinfection_enabled: true,
            },
        }
    }
}

/// Spawn initial disease
pub fn spawn_initial_disease(
    node_registry: Res<NodeRegistry>,
    mut spawn_writer: MessageWriter<ContagionSpawnRequested>,
) {
    // Select random city
    let mut rng = rand::thread_rng();
    let city_index = rng.gen_range(0..CITIES.len());
    let origin_city = &CITIES[city_index];

    let origin_entity = node_registry
        .get(origin_city.id)
        .expect("Origin city should exist");

    info!("Spawning Virus X in {}", origin_city.name);

    spawn_writer.write(ContagionSpawnRequested {
        contagion_id: "virus_x_1".to_string(),
        content: ContagionContent::Disease {
            severity: DiseaseLevel::Moderate,
            location: origin_city.id.to_string(),
        },
        origin_node: origin_entity,
        mutation_rate: 0.15,
    });
}

/// Get severity display
pub fn get_severity_display(severity: &DiseaseLevel) -> &'static str {
    match severity {
        DiseaseLevel::Mild => "Mild",
        DiseaseLevel::Moderate => "Moderate",
        DiseaseLevel::Severe => "Severe",
        DiseaseLevel::Critical => "Critical",
    }
}
