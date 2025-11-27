//! Unit tests for contagion propagation

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::super::{components::*, events::*, plugin::ContagionPlugin, resources::*};
    use crate::IssunCorePlugin;

    // ==================== Test Setup Helpers ====================

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            IssunCorePlugin,
            ContagionPlugin::default().with_seed(42),
        ));
        app
    }

    fn create_test_app_with_config(config: ContagionConfig) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            IssunCorePlugin,
            ContagionPlugin::default().with_seed(42).with_config(config),
        ));
        app
    }

    fn setup_basic_network(app: &mut App) -> (Entity, Entity, Entity) {
        let node_a = app
            .world_mut()
            .spawn(ContagionNode::new("node_a", NodeType::City, 10000))
            .id();

        let node_b = app
            .world_mut()
            .spawn(ContagionNode::new("node_b", NodeType::City, 8000))
            .id();

        let edge = app
            .world_mut()
            .spawn(PropagationEdge::new("edge_ab", node_a, node_b, 1.0))
            .id();

        // Register nodes
        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register("node_a", node_a);
        registry.register("node_b", node_b);

        (node_a, node_b, edge)
    }

    // ==================== Test: Spawn Contagion ====================

    #[test]
    fn test_spawn_contagion() {
        let mut app = create_test_app();
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        // Spawn contagion
        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "disease_1".to_string(),
            content: ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "node_a".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.1,
        });

        app.update();

        // Verify contagion entity created
        let mut query = app.world_mut().query::<&Contagion>();
        let contagions = query.iter(app.world()).count();
        assert_eq!(contagions, 1, "Should spawn 1 contagion");

        // Verify initial infection created
        let infections = app
            .world()
            .iter_entities()
            .filter(|e| e.contains::<ContagionInfection>())
            .count();
        assert_eq!(infections, 1, "Should create initial infection");

        // Verify infection is in Incubating state
        let infection = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<ContagionInfection>())
            .unwrap();
        assert!(
            matches!(infection.state, InfectionState::Incubating { .. }),
            "Initial infection should be Incubating"
        );

        // Verify spawned event
        let messages = app.world().resource::<Messages<ContagionSpawnedEvent>>();
        let mut cursor = messages.get_cursor();
        let spawned_events: Vec<_> = cursor.read(messages).cloned().collect();
        assert_eq!(spawned_events.len(), 1, "Should emit ContagionSpawnedEvent");
    }

    // ==================== Test: State Progression ====================

    #[test]
    fn test_state_progression_turn_based() {
        let config = ContagionConfig {
            time_mode: TimeMode::TurnBased,
            default_incubation_duration: DurationConfig::new(2.0, 0.0),
            default_active_duration: DurationConfig::new(3.0, 0.0),
            default_immunity_duration: DurationConfig::new(2.0, 0.0),
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        // Spawn contagion
        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "disease_1".to_string(),
            content: ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "node_a".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        // Helper to get infection state
        let get_state = |app: &App| -> InfectionState {
            app.world()
                .iter_entities()
                .find_map(|e| e.get::<ContagionInfection>())
                .unwrap()
                .state
                .clone()
        };

        // Turn 0: Incubating
        assert!(matches!(get_state(&app), InfectionState::Incubating { .. }));

        // Turn 1: Still Incubating
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();
        assert!(matches!(get_state(&app), InfectionState::Incubating { .. }));

        // Turn 2: Transition to Active
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();
        assert!(matches!(get_state(&app), InfectionState::Active { .. }));

        // Turn 3-4: Still Active
        for _ in 0..2 {
            app.world_mut().write_message(TurnAdvancedMessage);
            app.update();
        }
        assert!(matches!(get_state(&app), InfectionState::Active { .. }));

        // Turn 5: Transition to Recovered
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();
        assert!(matches!(get_state(&app), InfectionState::Recovered { .. }));

        // Turn 6: Still Recovered
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();
        assert!(matches!(get_state(&app), InfectionState::Recovered { .. }));

        // Turn 7: Transition to Plain
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();
        assert!(matches!(get_state(&app), InfectionState::Plain));
    }

    // ==================== Test: State-Based Transmission Rates ====================

    #[test]
    fn test_state_based_transmission_rates() {
        let config = ContagionConfig {
            time_mode: TimeMode::TurnBased,
            global_propagation_rate: 1.0,
            incubation_transmission_rate: 0.2,
            active_transmission_rate: 0.8,
            recovered_transmission_rate: 0.05,
            plain_transmission_rate: 0.0,
            default_incubation_duration: DurationConfig::new(1.0, 0.0),
            default_active_duration: DurationConfig::new(1.0, 0.0),
            default_immunity_duration: DurationConfig::new(1.0, 0.0),
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);

        // Create network: A → B → C → D
        let node_a = app
            .world_mut()
            .spawn(ContagionNode::new("a", NodeType::City, 1000))
            .id();
        let node_b = app
            .world_mut()
            .spawn(ContagionNode::new("b", NodeType::City, 1000))
            .id();
        let node_c = app
            .world_mut()
            .spawn(ContagionNode::new("c", NodeType::City, 1000))
            .id();
        let node_d = app
            .world_mut()
            .spawn(ContagionNode::new("d", NodeType::City, 1000))
            .id();

        app.world_mut()
            .spawn(PropagationEdge::new("ab", node_a, node_b, 1.0));
        app.world_mut()
            .spawn(PropagationEdge::new("bc", node_b, node_c, 1.0));
        app.world_mut()
            .spawn(PropagationEdge::new("cd", node_c, node_d, 1.0));

        let mut registry = app.world_mut().resource_mut::<NodeRegistry>();
        registry.register("a", node_a);
        registry.register("b", node_b);
        registry.register("c", node_c);
        registry.register("d", node_d);
        drop(registry);

        // Spawn at node A
        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "test".to_string(),
            content: ContagionContent::Custom {
                key: "test".to_string(),
                data: "data".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        // Count infected nodes
        let count_infected = |app: &App| -> usize {
            app.world()
                .iter_entities()
                .filter(|e| e.contains::<ContagionInfection>())
                .count()
        };

        // Initially: 1 infected (A in Incubating)
        assert_eq!(count_infected(&app), 1);

        // Propagation in Incubating state (rate=0.2, may not spread)
        app.world_mut().write_message(PropagationStepRequested);
        app.update();
        let count_after_incubating = count_infected(&app);

        // Advance to Active state
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();

        // Propagation in Active state (rate=0.8, likely spreads)
        app.world_mut().write_message(PropagationStepRequested);
        app.update();
        let count_after_active = count_infected(&app);

        // Active transmission should be more effective than Incubating
        assert!(
            count_after_active >= count_after_incubating,
            "Active state should transmit at least as much as Incubating"
        );
    }

    // ==================== Test: Time Modes ====================

    #[test]
    fn test_tick_based_time_mode() {
        let config = ContagionConfig {
            time_mode: TimeMode::TickBased,
            default_incubation_duration: DurationConfig::new(2.0, 0.0), // 120 ticks
            default_active_duration: DurationConfig::new(1.0, 0.0),     // 60 ticks
            default_immunity_duration: DurationConfig::new(1.0, 0.0),   // 60 ticks
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "test".to_string(),
            content: ContagionContent::Custom {
                key: "test".to_string(),
                data: "data".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        // Should start in Incubating
        let infection = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<ContagionInfection>())
            .unwrap();
        assert!(matches!(infection.state, InfectionState::Incubating { .. }));

        // Progress through ticks (120 ticks for incubation)
        for _ in 0..120 {
            app.update();
        }

        // Should now be Active
        let infection = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<ContagionInfection>())
            .unwrap();
        assert!(matches!(infection.state, InfectionState::Active { .. }));
    }

    // ==================== Test: Mutation ====================

    #[test]
    fn test_mutation() {
        let config = ContagionConfig {
            time_mode: TimeMode::TurnBased,
            global_propagation_rate: 1.0,
            active_transmission_rate: 1.0,
            default_incubation_duration: DurationConfig::new(0.0, 0.0),
            default_active_duration: DurationConfig::new(10.0, 0.0),
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);
        let (node_a, _node_b, edge) = setup_basic_network(&mut app);

        // Set edge noise level to enable mutations
        let mut edge_mut = app.world_mut().entity_mut(edge);
        edge_mut.insert(PropagationEdge {
            edge_id: "edge_ab".to_string(),
            from_node: node_a,
            to_node: _node_b,
            transmission_rate: 1.0,
            noise_level: 1.0, // Enable mutations
        });
        drop(edge_mut);

        // Spawn with high mutation rate
        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "disease_1".to_string(),
            content: ContagionContent::Disease {
                severity: DiseaseLevel::Mild,
                location: "node_a".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 1.0, // 100% mutation
        });
        app.update();

        // Advance turn to get disease into Active state for higher transmission rate
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();

        // Trigger propagation multiple times to increase mutation chance
        for _ in 0..50 {
            app.world_mut().write_message(PropagationStepRequested);
            app.update();
        }

        // Check if mutation occurred
        let messages = app.world().resource::<Messages<ContagionSpreadEvent>>();
        let mut cursor = messages.get_cursor();
        let spread_events: Vec<_> = cursor
            .read(messages)
            .filter(|e| e.is_mutation)
            .cloned()
            .collect();

        assert!(
            !spread_events.is_empty(),
            "Should have at least one mutation event"
        );

        // Check that mutated contagion has different ID
        let mutations_with_original = spread_events
            .iter()
            .filter(|e| e.original_id.is_some())
            .count();
        assert!(
            mutations_with_original > 0,
            "Mutations should reference original ID"
        );
    }

    // ==================== Test: Credibility Decay ====================

    #[test]
    fn test_credibility_decay() {
        let config = ContagionConfig {
            lifetime_turns: 10,
            min_credibility: 0.2,
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "rumor_1".to_string(),
            content: ContagionContent::Political {
                faction: "FactionA".to_string(),
                claim: "Test claim".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        // Initial credibility should be 1.0
        let initial_credibility = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<Contagion>())
            .unwrap()
            .credibility;
        assert_eq!(initial_credibility, 1.0);

        // Apply decay over 5 turns
        app.world_mut()
            .write_message(CredibilityDecayRequested { elapsed_turns: 5 });
        app.update();

        let after_decay = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<Contagion>())
            .unwrap()
            .credibility;
        assert!(after_decay < 1.0, "Credibility should decay");
        assert!(after_decay > 0.0, "Credibility should not reach zero yet");

        // Apply decay to expire contagion
        app.world_mut()
            .write_message(CredibilityDecayRequested { elapsed_turns: 10 });
        app.update();

        // Contagion should be removed
        let contagions = app
            .world()
            .iter_entities()
            .filter(|e| e.contains::<Contagion>())
            .count();
        assert_eq!(
            contagions, 0,
            "Contagion should be removed after full decay"
        );

        // Check removal event
        let messages = app.world().resource::<Messages<ContagionRemovedEvent>>();
        let mut cursor = messages.get_cursor();
        let removal_events: Vec<_> = cursor.read(messages).cloned().collect();
        assert_eq!(removal_events.len(), 1, "Should emit removal event");
    }

    // ==================== Test: Reinfection ====================

    #[test]
    fn test_reinfection_disabled() {
        let config = ContagionConfig {
            time_mode: TimeMode::TurnBased,
            default_incubation_duration: DurationConfig::new(0.0, 0.0),
            default_active_duration: DurationConfig::new(0.0, 0.0),
            default_immunity_duration: DurationConfig::new(0.0, 0.0),
            default_reinfection_enabled: false,
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        // Spawn contagion
        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "disease_1".to_string(),
            content: ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "node_a".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        // Progress to Plain state
        for _ in 0..5 {
            app.world_mut().write_message(TurnAdvancedMessage);
            app.update();
        }

        // Verify in Plain state
        let infection = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<ContagionInfection>())
            .unwrap();
        assert!(matches!(infection.state, InfectionState::Plain));

        // Verify reinfection is disabled
        let contagion = app
            .world()
            .iter_entities()
            .find_map(|e| e.get::<Contagion>())
            .unwrap();
        assert!(!contagion.reinfection_enabled);
    }

    // ==================== Test: Deterministic RNG ====================

    #[test]
    fn test_deterministic_rng() {
        // Run simulation twice with same seed
        let run_simulation = || -> Vec<String> {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                IssunCorePlugin,
                ContagionPlugin::default().with_seed(12345),
            ));

            let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

            app.world_mut().write_message(ContagionSpawnRequested {
                contagion_id: "test".to_string(),
                content: ContagionContent::Disease {
                    severity: DiseaseLevel::Moderate,
                    location: "node_a".to_string(),
                },
                origin_node: node_a,
                mutation_rate: 0.5,
            });
            app.update();

            // Run multiple propagation steps
            for _ in 0..5 {
                app.world_mut().write_message(PropagationStepRequested);
                app.update();
            }

            // Collect event IDs
            let messages = app.world().resource::<Messages<ContagionSpreadEvent>>();
            let mut cursor = messages.get_cursor();
            cursor
                .read(messages)
                .map(|e| e.contagion_id.clone())
                .collect()
        };

        let results1 = run_simulation();
        let results2 = run_simulation();

        assert_eq!(
            results1, results2,
            "Same seed should produce identical results"
        );
    }

    // ==================== Test: Entity Deletion Handling ====================

    #[test]
    fn test_entity_deletion_handling() {
        let mut app = create_test_app();
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        // Spawn contagion
        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "disease_1".to_string(),
            content: ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "node_a".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        let contagion_entity = app
            .world()
            .iter_entities()
            .find(|e| e.contains::<Contagion>())
            .unwrap()
            .id();

        // Delete contagion entity
        app.world_mut().despawn(contagion_entity);

        // Run systems - should not panic
        app.world_mut().write_message(PropagationStepRequested);
        app.update();
        app.world_mut().write_message(TurnAdvancedMessage);
        app.update();

        // Systems should handle missing entities gracefully
        // (no assertions needed - test passes if no panic)
    }

    // ==================== Test: State Change Events ====================

    #[test]
    fn test_state_change_events() {
        let config = ContagionConfig {
            time_mode: TimeMode::TurnBased,
            default_incubation_duration: DurationConfig::new(1.0, 0.0),
            default_active_duration: DurationConfig::new(1.0, 0.0),
            default_immunity_duration: DurationConfig::new(1.0, 0.0),
            ..Default::default()
        };

        let mut app = create_test_app_with_config(config);
        let (node_a, _node_b, _edge) = setup_basic_network(&mut app);

        app.world_mut().write_message(ContagionSpawnRequested {
            contagion_id: "disease_1".to_string(),
            content: ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "node_a".to_string(),
            },
            origin_node: node_a,
            mutation_rate: 0.0,
        });
        app.update();

        // Clear initial events
        let messages = app
            .world()
            .resource::<Messages<InfectionStateChangedEvent>>();
        let mut cursor = messages.get_cursor();
        let _: Vec<_> = cursor.read(messages).cloned().collect();

        // Trigger state transitions
        for _ in 0..3 {
            app.world_mut().write_message(TurnAdvancedMessage);
            app.update();
        }

        // Collect state change events
        let messages = app
            .world()
            .resource::<Messages<InfectionStateChangedEvent>>();
        let mut cursor = messages.get_cursor();
        let state_changes: Vec<_> = cursor.read(messages).cloned().collect();

        // Should have transitions: Incubating→Active, Active→Recovered, Recovered→Plain
        assert_eq!(state_changes.len(), 3, "Should emit 3 state change events");

        // Verify transition sequence
        assert_eq!(state_changes[0].old_state, InfectionStateType::Incubating);
        assert_eq!(state_changes[0].new_state, InfectionStateType::Active);
        assert_eq!(state_changes[1].old_state, InfectionStateType::Active);
        assert_eq!(state_changes[1].new_state, InfectionStateType::Recovered);
        assert_eq!(state_changes[2].old_state, InfectionStateType::Recovered);
        assert_eq!(state_changes[2].new_state, InfectionStateType::Plain);
    }
}
