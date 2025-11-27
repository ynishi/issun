//! System implementations for contagion propagation

use bevy::prelude::*;
use rand::Rng;

use super::{components::*, events::*, resources::*};

// ==================== Spawn System ====================

/// Spawn contagion entities
pub fn handle_contagion_spawn(
    mut commands: Commands,
    mut messages: MessageReader<ContagionSpawnRequested>,
    mut spawned_messages: MessageWriter<ContagionSpawnedEvent>,
    _node_registry: Res<NodeRegistry>,
    config: Res<ContagionConfig>,
    mut rng: ResMut<ContagionRng>,
) {
    for msg in messages.read() {
        let origin_entity = msg.origin_node;

        // Generate durations with variance
        let incubation_duration = generate_duration(
            &config.default_incubation_duration,
            &config.time_mode,
            &mut rng.rng,
        );
        let active_duration = generate_duration(
            &config.default_active_duration,
            &config.time_mode,
            &mut rng.rng,
        );
        let immunity_duration = generate_duration(
            &config.default_immunity_duration,
            &config.time_mode,
            &mut rng.rng,
        );

        // Spawn contagion entity
        let contagion_entity = commands
            .spawn(Contagion {
                contagion_id: msg.contagion_id.clone(),
                content: msg.content.clone(),
                mutation_rate: msg.mutation_rate,
                credibility: 1.0,
                origin_node: origin_entity,
                created_at: ContagionDuration::zero(&config.time_mode),
                incubation_duration,
                active_duration,
                immunity_duration,
                reinfection_enabled: config.default_reinfection_enabled,
            })
            .id();

        // Spawn initial infection at origin
        commands.spawn(ContagionInfection {
            contagion_entity,
            node_entity: origin_entity,
            state: InfectionState::Incubating {
                elapsed: ContagionDuration::zero(&config.time_mode),
                total_duration: incubation_duration,
            },
            infected_at: ContagionDuration::zero(&config.time_mode),
        });

        spawned_messages.write(ContagionSpawnedEvent {
            contagion_entity,
            contagion_id: msg.contagion_id.clone(),
            origin_node: origin_entity,
        });
    }
}

// ==================== State Progression Systems ====================

/// Progress infection states (Tick/Time-based)
pub fn progress_infection_states_continuous(
    infections: Query<(Entity, &mut ContagionInfection)>,
    contagions: Query<&Contagion>,
    state_changed: MessageWriter<InfectionStateChangedEvent>,
    config: Res<ContagionConfig>,
    time: Res<Time>,
) {
    if config.time_mode == TimeMode::TurnBased {
        return;
    }

    let delta = match config.time_mode {
        TimeMode::TickBased => ContagionDuration::Ticks(1),
        TimeMode::TimeBased => ContagionDuration::Seconds(time.delta_secs()),
        _ => return,
    };

    progress_infections_impl(
        infections,
        contagions,
        state_changed,
        delta,
        &config.time_mode,
    );
}

/// Progress infection states (Turn-based)
pub fn progress_infection_states_turn_based(
    mut turn_messages: MessageReader<TurnAdvancedMessage>,
    infections: Query<(Entity, &mut ContagionInfection)>,
    contagions: Query<&Contagion>,
    state_changed: MessageWriter<InfectionStateChangedEvent>,
    config: Res<ContagionConfig>,
) {
    // Process all turn messages (should be at most one per frame)
    let has_message = turn_messages.read().next().is_some();
    if has_message {
        let delta = ContagionDuration::Turns(1);
        progress_infections_impl(
            infections,
            contagions,
            state_changed,
            delta,
            &config.time_mode,
        );
    }
}

fn progress_infections_impl(
    mut infections: Query<(Entity, &mut ContagionInfection)>,
    contagions: Query<&Contagion>,
    mut state_changed: MessageWriter<InfectionStateChangedEvent>,
    delta: ContagionDuration,
    time_mode: &TimeMode,
) {
    for (infection_entity, mut infection) in infections.iter_mut() {
        let Ok(contagion) = contagions.get(infection.contagion_entity) else {
            continue;
        };

        let old_state = infection.state.get_type();
        let mut transitioned = false;

        match &mut infection.state {
            InfectionState::Incubating {
                elapsed,
                total_duration,
            } => {
                elapsed.add(&delta);
                if total_duration.is_expired(elapsed) {
                    infection.state = InfectionState::Active {
                        elapsed: ContagionDuration::zero(time_mode),
                        total_duration: contagion.active_duration,
                    };
                    transitioned = true;
                }
            }
            InfectionState::Active {
                elapsed,
                total_duration,
            } => {
                elapsed.add(&delta);
                if total_duration.is_expired(elapsed) {
                    infection.state = InfectionState::Recovered {
                        elapsed: ContagionDuration::zero(time_mode),
                        immunity_duration: contagion.immunity_duration,
                    };
                    transitioned = true;
                }
            }
            InfectionState::Recovered {
                elapsed,
                immunity_duration,
            } => {
                elapsed.add(&delta);
                if immunity_duration.is_expired(elapsed) {
                    infection.state = InfectionState::Plain;
                    transitioned = true;
                }
            }
            InfectionState::Plain => {}
        }

        if transitioned {
            let new_state = infection.state.get_type();
            state_changed.write(InfectionStateChangedEvent {
                infection_entity,
                node_entity: infection.node_entity,
                contagion_id: contagion.contagion_id.clone(),
                old_state,
                new_state,
            });
        }
    }
}

// ==================== Propagation System ====================

/// Main propagation system (with state-based transmission rates)
#[allow(clippy::too_many_arguments)]
pub fn handle_propagation_step(
    mut commands: Commands,
    mut messages: MessageReader<PropagationStepRequested>,
    mut spread_messages: MessageWriter<ContagionSpreadEvent>,
    mut completed_messages: MessageWriter<PropagationStepCompletedEvent>,
    config: Res<ContagionConfig>,
    mut rng: ResMut<ContagionRng>,
    contagions: Query<(Entity, &Contagion)>,
    infections: Query<&ContagionInfection>,
    nodes: Query<&ContagionNode>,
    edges: Query<&PropagationEdge>,
) {
    for _msg in messages.read() {
        let mut spread_count = 0;
        let mut mutation_count = 0;
        let mut pending_spreads = Vec::new();

        for infection in infections.iter() {
            // Get state-based transmission rate
            let state_transmission_rate = match &infection.state {
                InfectionState::Incubating { .. } => config.incubation_transmission_rate,
                InfectionState::Active { .. } => config.active_transmission_rate,
                InfectionState::Recovered { .. } => config.recovered_transmission_rate,
                InfectionState::Plain => config.plain_transmission_rate,
            };

            if state_transmission_rate == 0.0 {
                continue;
            }

            let Ok((contagion_entity, contagion)) = contagions.get(infection.contagion_entity)
            else {
                continue;
            };

            // Check all outgoing edges
            for edge in edges.iter() {
                if edge.from_node != infection.node_entity {
                    continue;
                }

                // Check if target already infected
                let target_infected = infections.iter().any(|i| {
                    i.node_entity == edge.to_node && i.contagion_entity == contagion_entity
                });
                if target_infected {
                    continue;
                }

                let Ok(target_node) = nodes.get(edge.to_node) else {
                    continue;
                };

                // Calculate propagation chance
                let propagation_chance = edge.transmission_rate
                    * config.global_propagation_rate
                    * state_transmission_rate
                    * contagion.credibility
                    * (1.0 - target_node.resistance);

                if rng.rng.gen::<f32>() < propagation_chance {
                    let mutation_chance = contagion.mutation_rate * edge.noise_level;
                    let is_mutation = rng.rng.gen::<f32>() < mutation_chance;

                    pending_spreads.push(PendingSpread {
                        contagion_entity,
                        contagion: contagion.clone(),
                        from_node: infection.node_entity,
                        to_node: edge.to_node,
                        is_mutation,
                    });
                }
            }
        }

        // Apply spreads
        for pending in pending_spreads {
            let (target_contagion, target_contagion_id) = if pending.is_mutation {
                // Create mutated contagion
                let mutated_content = mutate_content(&pending.contagion.content, &mut rng.rng);
                let mutated_id = format!(
                    "{}_{}",
                    pending.contagion.contagion_id,
                    rng.rng.gen::<u64>()
                );

                let mutated = commands
                    .spawn(Contagion {
                        contagion_id: mutated_id.clone(),
                        content: mutated_content,
                        mutation_rate: pending.contagion.mutation_rate,
                        credibility: pending.contagion.credibility * 0.9,
                        origin_node: pending.to_node,
                        created_at: pending.contagion.created_at,
                        incubation_duration: pending.contagion.incubation_duration,
                        active_duration: pending.contagion.active_duration,
                        immunity_duration: pending.contagion.immunity_duration,
                        reinfection_enabled: pending.contagion.reinfection_enabled,
                    })
                    .id();

                mutation_count += 1;
                (mutated, mutated_id)
            } else {
                (
                    pending.contagion_entity,
                    pending.contagion.contagion_id.clone(),
                )
            };

            // Spawn infection at target node
            let infection_entity = commands
                .spawn(ContagionInfection {
                    contagion_entity: target_contagion,
                    node_entity: pending.to_node,
                    state: InfectionState::Incubating {
                        elapsed: ContagionDuration::zero(&config.time_mode),
                        total_duration: pending.contagion.incubation_duration,
                    },
                    infected_at: ContagionDuration::zero(&config.time_mode),
                })
                .id();

            spread_messages.write(ContagionSpreadEvent {
                infection_entity,
                contagion_id: target_contagion_id.clone(),
                from_node: pending.from_node,
                to_node: pending.to_node,
                is_mutation: pending.is_mutation,
                original_id: if pending.is_mutation {
                    Some(pending.contagion.contagion_id.clone())
                } else {
                    None
                },
            });

            spread_count += 1;
        }

        completed_messages.write(PropagationStepCompletedEvent {
            spread_count,
            mutation_count,
        });
    }
}

struct PendingSpread {
    contagion_entity: Entity,
    contagion: Contagion,
    from_node: Entity,
    to_node: Entity,
    is_mutation: bool,
}

// ==================== Credibility Decay System ====================

/// Credibility decay system
pub fn handle_credibility_decay(
    mut commands: Commands,
    mut messages: MessageReader<CredibilityDecayRequested>,
    mut removed_messages: MessageWriter<ContagionRemovedEvent>,
    config: Res<ContagionConfig>,
    mut contagions: Query<(Entity, &mut Contagion)>,
    infections: Query<(Entity, &ContagionInfection)>,
) {
    for msg in messages.read() {
        let decay_rate = 1.0 / config.lifetime_turns as f32;
        let total_decay = decay_rate * msg.elapsed_turns as f32;

        let mut to_remove = Vec::new();

        for (entity, mut contagion) in contagions.iter_mut() {
            contagion.credibility = (contagion.credibility - total_decay).max(0.0);

            if contagion.credibility < config.min_credibility {
                to_remove.push((entity, contagion.contagion_id.clone()));
            }
        }

        for (contagion_entity, contagion_id) in to_remove {
            // Remove contagion
            commands.entity(contagion_entity).despawn();

            // Remove all associated infections
            for (infection_entity, infection) in infections.iter() {
                if infection.contagion_entity == contagion_entity {
                    commands.entity(infection_entity).despawn();
                }
            }

            removed_messages.write(ContagionRemovedEvent {
                contagion_id,
                reason: RemovalReason::Expired,
            });
        }
    }
}

// ==================== Helper Functions ====================

fn mutate_content(content: &ContagionContent, rng: &mut impl Rng) -> ContagionContent {
    match content {
        ContagionContent::Disease { severity, location } => {
            let new_severity = if rng.gen::<f32>() < 0.7 {
                severity.increase()
            } else {
                severity.decrease()
            };
            ContagionContent::Disease {
                severity: new_severity,
                location: location.clone(),
            }
        }
        ContagionContent::ProductReputation { product, sentiment } => {
            ContagionContent::ProductReputation {
                product: product.clone(),
                sentiment: (sentiment * 1.5).clamp(-1.0, 1.0),
            }
        }
        ContagionContent::Political { faction, claim } => ContagionContent::Political {
            faction: faction.clone(),
            claim: format!("{} (exaggerated)", claim),
        },
        ContagionContent::MarketTrend {
            commodity,
            direction,
        } => {
            let new_direction = if rng.gen::<f32>() < 0.7 {
                *direction
            } else {
                match direction {
                    TrendDirection::Bullish => TrendDirection::Bearish,
                    TrendDirection::Bearish => TrendDirection::Bullish,
                    TrendDirection::Neutral => TrendDirection::Neutral,
                }
            };
            ContagionContent::MarketTrend {
                commodity: commodity.clone(),
                direction: new_direction,
            }
        }
        ContagionContent::Custom { key, data } => ContagionContent::Custom {
            key: key.clone(),
            data: data.clone(),
        },
    }
}

fn generate_duration(
    config: &DurationConfig,
    mode: &TimeMode,
    rng: &mut impl Rng,
) -> ContagionDuration {
    let variance_factor = 1.0 + (rng.gen::<f32>() - 0.5) * config.variance;
    let value = (config.base * variance_factor).max(1.0);

    match mode {
        TimeMode::TurnBased => ContagionDuration::Turns(value as u64),
        TimeMode::TickBased => ContagionDuration::Ticks((value * 60.0) as u64),
        TimeMode::TimeBased => ContagionDuration::Seconds(value),
    }
}
