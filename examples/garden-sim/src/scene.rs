//! Scene definitions for Garden Simulator

use crate::models::PlantSpecies;
use issun::prelude::*;
use issun::ui::InputEvent;
use issun::Scene;
use serde::{Deserialize, Serialize};

/// Game scene enum
#[derive(Debug, Clone, Serialize, Deserialize, Scene)]
#[scene(handler_params = "input: ::issun::ui::InputEvent")]
pub enum GameScene {
    Simulation(SimulationSceneData),
}

/// Simulation scene data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSceneData {
    pub paused: bool,
    pub tick_count: u64,
    pub plants: Vec<PlantSpecies>,
}

impl SimulationSceneData {
    pub fn new() -> Self {
        Self {
            paused: false,
            tick_count: 0,
            plants: vec![
                PlantSpecies::Tomato,
                PlantSpecies::Lettuce,
                PlantSpecies::Carrot,
                PlantSpecies::Wheat,
                PlantSpecies::Sunflower,
            ],
        }
    }

    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Cancel => SceneTransition::Quit,
            InputEvent::Char(' ') => {
                // Toggle pause with Space
                self.paused = !self.paused;
                SceneTransition::Stay
            }
            _ => SceneTransition::Stay,
        }
    }
}

impl Default for SimulationSceneData {
    fn default() -> Self {
        Self::new()
    }
}
