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
    pub scroll_offset: usize,
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
            scroll_offset: 0,
        }
    }

    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Cancel => SceneTransition::Quit,
            InputEvent::Char(' ') => {
                // Toggle pause with Space
                self.paused = !self.paused;
                SceneTransition::Stay
            }
            InputEvent::Up => {
                // Scroll up
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Down => {
                // Scroll down (will be clamped in UI rendering)
                if let Some(garden) = resources.try_get::<crate::garden::Garden>() {
                    let max_offset = garden.plants.len().saturating_sub(1);
                    if self.scroll_offset < max_offset {
                        self.scroll_offset += 1;
                    }
                }
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
