//! Combat systems for processing combat logic

use bevy::log::{info, warn};
use bevy::prelude::*;

use super::components::*;
use super::events::*;

// =============================================================================
// Combat Start System
// =============================================================================

/// Handle combat start requests
pub fn handle_combat_start(
    mut commands: Commands,
    mut requests: MessageReader<CombatStartRequested>,
    mut started_events: MessageWriter<CombatStartedEvent>,
    config: Res<CombatConfig>,
) {
    for request in requests.read() {
        // Generate seed from battle_id for deterministic replay
        let seed = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            request.battle_id.hash(&mut hasher);
            hasher.finish()
        };

        // Spawn combat session entity
        let combat_entity = commands
            .spawn((
                CombatSession::new(request.battle_id.clone()),
                CombatParticipants::new(vec![]), // Will be populated separately
                CombatLog::new(config.max_log_entries),
                ReplayRecorder::new(),
                UniqueId(request.battle_id.clone()), // Use battle_id as stable ID
                CombatSessionRng::new(seed),         // Seeded RNG for replay
            ))
            .id();

        // Emit started event
        started_events.write(CombatStartedEvent {
            battle_id: request.battle_id.clone(),
            combat_entity,
        });

        info!("Combat started: {}", request.battle_id);
    }
}

// =============================================================================
// Turn Advance System
// =============================================================================

/// Handle turn advance requests
pub fn handle_turn_advance(
    mut requests: MessageReader<CombatTurnAdvanceRequested>,
    mut sessions: Query<(&mut CombatSession, &mut CombatLog)>,
    mut completed_events: MessageWriter<CombatTurnCompletedEvent>,
) {
    for request in requests.read() {
        // Find combat session by battle_id
        for (mut session, log) in sessions.iter_mut() {
            if session.battle_id == request.battle_id {
                session.turn_count += 1;

                let log_entries = if log.entries.is_empty() {
                    vec![]
                } else {
                    log.entries.iter().map(|e| e.message.clone()).collect()
                };

                completed_events.write(CombatTurnCompletedEvent {
                    battle_id: request.battle_id.clone(),
                    turn: session.turn_count,
                    log_entries,
                });

                info!(
                    "Turn {} completed for {}",
                    session.turn_count, request.battle_id
                );
            }
        }
    }
}

// =============================================================================
// Combat End System
// =============================================================================

/// Handle combat end requests
pub fn handle_combat_end(
    mut commands: Commands,
    mut requests: MessageReader<CombatEndRequested>,
    sessions: Query<(Entity, &CombatSession)>,
    mut ended_events: MessageWriter<CombatEndedEvent>,
) {
    for request in requests.read() {
        // Find combat session by battle_id
        for (entity, session) in sessions.iter() {
            if session.battle_id == request.battle_id {
                ended_events.write(CombatEndedEvent {
                    battle_id: request.battle_id.clone(),
                    result: CombatResult::Ongoing, // TODO: Determine actual result
                    total_turns: session.turn_count,
                    score: session.score,
                });

                // Despawn combat session entity
                commands.entity(entity).despawn();

                info!("Combat ended: {}", request.battle_id);
            }
        }
    }
}

// =============================================================================
// Damage Processing System
// =============================================================================

/// Handle damage requests
///
/// ⚠️ CRITICAL: Uses Entity validation to prevent panics
pub fn handle_damage_request(
    mut requests: MessageReader<DamageRequested>,
    mut applied_events: MessageWriter<DamageAppliedEvent>,
    _attacks: Query<&Attack>,
    defenses: Query<&Defense>,
    mut healths: Query<&mut Health>,
    config: Res<CombatConfig>,
    mut sessions: Query<&mut CombatLog>,
) {
    for request in requests.read() {
        // ⚠️ CRITICAL: Entity validation required
        let Ok(mut target_health) = healths.get_mut(request.target) else {
            warn!("Target entity {:?} has no Health component", request.target);
            continue;
        };

        // Get defense value (optional)
        let defense_value = defenses.get(request.target).map(|d| d.value).unwrap_or(0);

        // Calculate damage
        let actual_damage = (request.base_damage - defense_value).max(config.min_damage);

        // Apply damage
        target_health.take_damage(actual_damage);
        let is_dead = !target_health.is_alive();

        // Emit event
        applied_events.write(DamageAppliedEvent {
            attacker: request.attacker,
            target: request.target,
            actual_damage,
            is_dead,
        });

        info!(
            "Damage applied: {} damage to {:?} (dead: {})",
            actual_damage, request.target, is_dead
        );

        // Add to combat log (find session containing this combat)
        for mut log in sessions.iter_mut() {
            log.add_entry(
                0, // TODO: Get current turn from session
                format!("{} damage dealt", actual_damage),
            );
        }
    }
}

// =============================================================================
// Cleanup System (Zombie Entity Removal)
// =============================================================================

/// Remove zombie entities from participant lists
pub fn cleanup_zombie_entities(
    mut participants: Query<&mut CombatParticipants>,
    combatants: Query<&Combatant>,
) {
    for mut participant_list in participants.iter_mut() {
        participant_list.cleanup_zombies(|entity| combatants.get(entity).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_start_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CombatConfig>()
            .add_message::<CombatStartRequested>()
            .add_message::<CombatStartedEvent>()
            .add_systems(Update, handle_combat_start);

        // Send combat start request
        app.world_mut().write_message(CombatStartRequested {
            battle_id: "test_battle".to_string(),
            combat_entity: Entity::PLACEHOLDER,
        });

        // Run one frame
        app.update();

        // Check that CombatStartedEvent was emitted
        let combat_entity = {
            let mut started_events = app
                .world_mut()
                .resource_mut::<Messages<CombatStartedEvent>>();
            let events: Vec<_> = started_events.drain().collect();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].battle_id, "test_battle");
            events[0].combat_entity
        };

        // Check that combat session was created
        let world = app.world();
        let session = world.get::<CombatSession>(combat_entity);
        assert!(session.is_some());
        assert_eq!(session.unwrap().battle_id, "test_battle");
    }

    #[test]
    fn test_damage_system_with_defense() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CombatConfig>()
            .add_message::<DamageRequested>()
            .add_message::<DamageAppliedEvent>()
            .add_systems(Update, handle_damage_request);

        // Spawn target with health and defense
        let target = app
            .world_mut()
            .spawn((Health::new(100), Defense { value: 10 }))
            .id();

        let attacker = app.world_mut().spawn(Attack { power: 30 }).id();

        // Request damage
        app.world_mut().write_message(DamageRequested {
            attacker,
            target,
            base_damage: 30,
        });

        // Run one frame
        app.update();

        // Check damage was applied
        let health = app.world().get::<Health>(target).unwrap();
        assert_eq!(health.current, 80); // 100 - (30 - 10) = 80

        // Check event was emitted
        let mut damage_events = app
            .world_mut()
            .resource_mut::<Messages<DamageAppliedEvent>>();
        let events: Vec<_> = damage_events.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].actual_damage, 20);
        assert!(!events[0].is_dead);
    }

    #[test]
    fn test_damage_system_minimum_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CombatConfig>()
            .add_message::<DamageRequested>()
            .add_message::<DamageAppliedEvent>()
            .add_systems(Update, handle_damage_request);

        // Target with very high defense
        let target = app
            .world_mut()
            .spawn((Health::new(100), Defense { value: 100 }))
            .id();

        let attacker = app.world_mut().spawn(Attack { power: 10 }).id();

        // Request damage
        app.world_mut().write_message(DamageRequested {
            attacker,
            target,
            base_damage: 10,
        });

        app.update();

        // Check minimum damage was applied
        let health = app.world().get::<Health>(target).unwrap();
        assert_eq!(health.current, 99); // 100 - 1 (min damage)
    }

    #[test]
    fn test_damage_on_deleted_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CombatConfig>()
            .add_message::<DamageRequested>()
            .add_message::<DamageAppliedEvent>()
            .add_systems(Update, handle_damage_request);

        let target = app.world_mut().spawn(Health::new(100)).id();
        let attacker = app.world_mut().spawn(Attack { power: 30 }).id();

        // Despawn target before damage request
        app.world_mut().entity_mut(target).despawn();

        // Request damage to deleted entity
        app.world_mut().write_message(DamageRequested {
            attacker,
            target,
            base_damage: 30,
        });

        // Should not panic
        app.update();

        // No damage event should be emitted
        let mut damage_events = app
            .world_mut()
            .resource_mut::<Messages<DamageAppliedEvent>>();
        let events: Vec<_> = damage_events.drain().collect();
        assert_eq!(events.len(), 0);
    }
}
