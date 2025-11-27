//! Game setup plugin - using issun-core and contagion_v2

use crate::components::*;
use crate::resources::{ContagionGraph, GameContext, UIState, VictoryResult};
use crate::states::GameScene;
use crate::systems::*;
use bevy::prelude::*;
use issun_bevy::plugins::contagion_v2::*;

/// Game setup plugin - initializes resources and entities
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        // Add contagion_v2 plugin first
        app.add_plugins(ContagionV2Plugin::new().with_base_rate(0.15));

        // Game states and resources
        app.init_state::<GameScene>()
            .insert_resource(GameContext::default())
            .insert_resource(ContagionGraph::build_city_topology())
            .insert_resource(VictoryResult::default())
            .insert_resource(UIState::default());

        // Setup systems
        app.add_systems(Startup, (setup_districts, setup_initial_infection))
            .add_systems(OnEnter(GameScene::Game), infect_initial_district);

        // Game logic systems
        // Note: contagion_step_system runs in IssunSet::Logic via ContagionV2Plugin
        app.add_systems(
            Update,
            (
                // 1. Propagate infection between districts (updates density)
                propagate_infection_between_districts_system,
                // 2. Sync ContagionState to District (after contagion_step_system)
                sync_contagion_to_district_system,
                // 3. Check win conditions
                check_win_condition_system,
            )
                .chain() // Run in sequence
                .run_if(in_state(GameScene::Game)),
        );
    }
}

fn setup_districts(mut commands: Commands) {
    let districts = vec![
        ("downtown", "Downtown", 10000),
        ("industrial", "Industrial Zone", 8000),
        ("residential", "Residential Area", 15000),
        ("suburbs", "Suburbs", 12000),
        ("harbor", "Harbor District", 9000),
    ];

    for (id, name, population) in districts {
        let resistance = 10; // Base resistance
        let density = 0.0; // Start with no infection pressure (will be updated by propagation)

        commands.spawn((
            District::new(id, name, population),
            ContagionState::<PlagueVirus>::default(),
            ContagionInputParams::new(density, resistance),
        ));
    }
}
