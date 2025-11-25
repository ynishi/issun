use crate::components::District;
use crate::resources::{ContagionGraph, GameContext, UIState, VictoryResult};
use crate::states::GameScene;
use crate::systems::*;
use bevy::prelude::*;

/// Game setup plugin - initializes resources and entities
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .insert_resource(GameContext::default())
            .insert_resource(ContagionGraph::build_city_topology())
            .insert_resource(VictoryResult::default())
            .insert_resource(UIState::default())
            .add_systems(Startup, (setup_districts, setup_initial_infection))
            .add_systems(OnEnter(GameScene::Game), infect_initial_district)
            .add_systems(
                Update,
                (
                    spread_contagion_system,
                    mutate_virus_system.after(spread_contagion_system),
                    check_win_condition_system.after(mutate_virus_system),
                )
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
        commands.spawn(District::new(id, name, population));
    }
}
