//! Integration tests for Action Plugin + Time Plugin
//!
//! Tests the interaction between ActionPlugin and TimePlugin,
//! focusing on turn advancement and action reset mechanics.

use bevy::prelude::*;
use issun_bevy::plugins::action::{
    ActionConsumedMessage, ActionPlugin, ActionPoints, ActionsResetMessage, CheckTurnEndMessage,
    ConsumeActionMessage,
};
use issun_bevy::plugins::time::{AdvanceTimeRequested, DayChanged, TimePlugin};
use issun_bevy::IssunCorePlugin;

/// Test full turn cycle: consume → deplete → advance → reset
#[test]
fn test_full_turn_cycle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    // Spawn single player
    let player = app
        .world_mut()
        .spawn((Name::new("Player"), ActionPoints::new(3)))
        .id();

    // Verify initial state
    {
        let points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(points.available, 3);
        assert_eq!(points.max_per_period, 3);
    }

    // Consume all actions
    for i in 1..=3 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player,
            context: format!("Action {}", i),
        });
        app.update();
    }

    // Verify player depleted
    {
        let points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(points.available, 0);
        assert!(points.is_depleted());
    }

    // Verify CheckTurnEndMessage published
    app.update(); // Process CheckTurnEndMessage
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            1,
            "AdvanceTimeRequested should be published after depletion"
        );
    }

    // Simulate day change (TimePlugin processes AdvanceTimeRequested)
    app.world_mut().write_message(DayChanged { day: 2 });
    app.update();

    // Verify action points reset
    {
        let points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(points.available, 3);
        assert!(!points.is_depleted());
    }

    // Verify reset message published
    {
        let mut reset_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionsResetMessage>>();
        let msgs: Vec<_> = reset_msgs.drain().collect();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].entity, player);
        assert_eq!(msgs[0].new_count, 3);
    }
}

/// Test multi-player turn advancement (ALL must deplete)
#[test]
fn test_multi_player_turn_advancement() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    // Spawn multiple players
    let player1 = app
        .world_mut()
        .spawn((Name::new("Player 1"), ActionPoints::new(3)))
        .id();

    let player2 = app
        .world_mut()
        .spawn((Name::new("Player 2"), ActionPoints::new(3)))
        .id();

    // Player 1 consumes all actions
    for i in 1..=3 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player1,
            context: format!("Player 1 Action {}", i),
        });
        app.update();
    }

    // Verify player1 depleted
    {
        let p1_points = app.world().get::<ActionPoints>(player1).unwrap();
        assert_eq!(p1_points.available, 0);
        assert!(p1_points.is_depleted());
    }

    // Player 2 still has actions
    {
        let p2_points = app.world().get::<ActionPoints>(player2).unwrap();
        assert_eq!(p2_points.available, 3);
    }

    // Verify CheckTurnEndMessage published but NOT AdvanceTimeRequested
    app.update();
    {
        let mut check_msgs = app
            .world_mut()
            .resource_mut::<Messages<CheckTurnEndMessage>>();
        let msgs: Vec<_> = check_msgs.drain().collect();
        assert!(
            !msgs.is_empty(),
            "CheckTurnEndMessage should be published when player1 depletes"
        );
    }

    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            0,
            "Turn should NOT advance when only one player is depleted"
        );
    }

    // Player 2 consumes all actions
    for i in 1..=3 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player2,
            context: format!("Player 2 Action {}", i),
        });
        app.update();
    }

    // Verify both players depleted
    {
        let p1_points = app.world().get::<ActionPoints>(player1).unwrap();
        assert!(p1_points.is_depleted());

        let p2_points = app.world().get::<ActionPoints>(player2).unwrap();
        assert!(p2_points.is_depleted());
    }

    // NOW AdvanceTimeRequested should be published
    app.update(); // Process CheckTurnEndMessage
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            1,
            "Turn SHOULD advance when ALL players are depleted"
        );
    }

    // Simulate day change
    app.world_mut().write_message(DayChanged { day: 2 });
    app.update();

    // Verify both players reset
    {
        let p1_points = app.world().get::<ActionPoints>(player1).unwrap();
        assert_eq!(p1_points.available, 3);

        let p2_points = app.world().get::<ActionPoints>(player2).unwrap();
        assert_eq!(p2_points.available, 3);
    }
}

/// Regression test: Turn does NOT advance when some players have actions
#[test]
fn test_turn_does_not_advance_prematurely() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    // Spawn 3 players with different action counts
    let player1 = app
        .world_mut()
        .spawn((Name::new("Player 1"), ActionPoints::new(2)))
        .id();

    let player2 = app
        .world_mut()
        .spawn((Name::new("Player 2"), ActionPoints::new(3)))
        .id();

    let player3 = app
        .world_mut()
        .spawn((Name::new("Player 3"), ActionPoints::new(1)))
        .id();

    // Player 1 depletes (2 actions)
    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player1,
            context: "Action".to_string(),
        });
        app.update();
    }

    // Player 3 depletes (1 action)
    app.world_mut().write_message(ConsumeActionMessage {
        entity: player3,
        context: "Action".to_string(),
    });
    app.update();

    // Player 2 still has 3 actions
    {
        let p2_points = app.world().get::<ActionPoints>(player2).unwrap();
        assert_eq!(p2_points.available, 3);
    }

    // Process CheckTurnEndMessage
    app.update();

    // Verify NO AdvanceTimeRequested published
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            0,
            "Turn should NOT advance when player2 still has 3 actions"
        );
    }

    // Player 2 consumes 1 action (still has 2 remaining)
    app.world_mut().write_message(ConsumeActionMessage {
        entity: player2,
        context: "Action".to_string(),
    });
    app.update();

    // Still should not advance
    app.update();
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            0,
            "Turn should NOT advance when player2 still has 2 actions"
        );
    }
}

/// Test handling of deleted entities during turn cycle
#[test]
fn test_deleted_entity_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    // Spawn 2 players
    let player1 = app
        .world_mut()
        .spawn((Name::new("Player 1"), ActionPoints::new(3)))
        .id();

    let player2 = app
        .world_mut()
        .spawn((Name::new("Player 2"), ActionPoints::new(3)))
        .id();

    // Player 1 consumes 1 action
    app.world_mut().write_message(ConsumeActionMessage {
        entity: player1,
        context: "Action".to_string(),
    });
    app.update();

    // Clear existing consumed messages from first action
    {
        let mut consumed_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionConsumedMessage>>();
        let _: Vec<_> = consumed_msgs.drain().collect();
    }

    // Delete player1
    app.world_mut().despawn(player1);

    // Try to consume action for deleted entity (should not panic)
    app.world_mut().write_message(ConsumeActionMessage {
        entity: player1,
        context: "Should be skipped".to_string(),
    });
    app.update();

    // Verify no consumed message for deleted entity
    {
        let mut consumed_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionConsumedMessage>>();
        let msgs: Vec<_> = consumed_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            0,
            "No consumed message should be published for deleted entity"
        );
    }

    // Player 2 depletes
    for _ in 0..3 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player2,
            context: "Action".to_string(),
        });
        app.update();
    }

    // Turn should advance (only player2 exists and is depleted)
    app.update();
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(
            msgs.len(),
            1,
            "Turn should advance when all existing entities are depleted"
        );
    }

    // Day change should not panic on deleted entity
    app.world_mut().write_message(DayChanged { day: 2 });
    app.update();

    // Verify player2 reset
    {
        let p2_points = app.world().get::<ActionPoints>(player2).unwrap();
        assert_eq!(p2_points.available, 3);
    }

    // Verify only 1 reset message (for player2, not deleted player1)
    {
        let mut reset_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionsResetMessage>>();
        let msgs: Vec<_> = reset_msgs.drain().collect();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].entity, player2);
    }
}

/// Test action consumption with different max_per_period values
#[test]
fn test_different_action_budgets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    // Spawn entities with different action budgets
    let player = app
        .world_mut()
        .spawn((Name::new("Player"), ActionPoints::new(3)))
        .id();

    let faction = app
        .world_mut()
        .spawn((Name::new("Faction"), ActionPoints::new(5)))
        .id();

    let npc = app
        .world_mut()
        .spawn((Name::new("NPC"), ActionPoints::new(2)))
        .id();

    // All consume different amounts
    // Player: 2/3
    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player,
            context: "Action".to_string(),
        });
        app.update();
    }

    // Faction: 3/5
    for _ in 0..3 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: faction,
            context: "Action".to_string(),
        });
        app.update();
    }

    // NPC: 2/2 (depleted)
    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: npc,
            context: "Action".to_string(),
        });
        app.update();
    }

    // Verify different remaining amounts
    {
        let player_points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(player_points.available, 1);

        let faction_points = app.world().get::<ActionPoints>(faction).unwrap();
        assert_eq!(faction_points.available, 2);

        let npc_points = app.world().get::<ActionPoints>(npc).unwrap();
        assert_eq!(npc_points.available, 0);
    }

    // Should NOT advance (player and faction still have actions)
    app.update();
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(msgs.len(), 0);
    }

    // Deplete remaining
    app.world_mut().write_message(ConsumeActionMessage {
        entity: player,
        context: "Action".to_string(),
    });
    app.update();

    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: faction,
            context: "Action".to_string(),
        });
        app.update();
    }

    // Now all depleted, should advance
    app.update();
    {
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(msgs.len(), 1);
    }

    // Day change resets all to different max values
    app.world_mut().write_message(DayChanged { day: 2 });
    app.update();

    {
        let player_points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(player_points.available, 3);

        let faction_points = app.world().get::<ActionPoints>(faction).unwrap();
        assert_eq!(faction_points.available, 5);

        let npc_points = app.world().get::<ActionPoints>(npc).unwrap();
        assert_eq!(npc_points.available, 2);
    }
}

/// Test sequential turn cycles (multiple turns)
#[test]
fn test_sequential_turn_cycles() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    let player = app
        .world_mut()
        .spawn((Name::new("Player"), ActionPoints::new(2)))
        .id();

    // Turn 1
    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player,
            context: "Turn 1 Action".to_string(),
        });
        app.update();
    }

    app.update(); // Advance
    app.world_mut().write_message(DayChanged { day: 2 });
    app.update();

    {
        let points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(points.available, 2);
    }

    // Turn 2
    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player,
            context: "Turn 2 Action".to_string(),
        });
        app.update();
    }

    app.update(); // Advance
    app.world_mut().write_message(DayChanged { day: 3 });
    app.update();

    {
        let points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(points.available, 2);
    }

    // Turn 3
    for _ in 0..2 {
        app.world_mut().write_message(ConsumeActionMessage {
            entity: player,
            context: "Turn 3 Action".to_string(),
        });
        app.update();
    }

    app.update(); // Advance
    app.world_mut().write_message(DayChanged { day: 4 });
    app.update();

    {
        let points = app.world().get::<ActionPoints>(player).unwrap();
        assert_eq!(points.available, 2);
    }
}
