use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Game mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Plague,
    Savior,
}

/// Game-wide context resource
#[derive(Resource, Debug, Clone)]
pub struct GameContext {
    pub turn: u32,
    pub max_turns: u32,
    pub mode: GameMode,
}

impl Default for GameContext {
    fn default() -> Self {
        Self {
            turn: 0,
            max_turns: 20,
            mode: GameMode::Plague,
        }
    }
}

/// Victory/defeat result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VictoryState {
    Victory(String),
    Defeat(String),
}

/// Victory result resource
#[derive(Resource, Debug, Clone, Default)]
pub struct VictoryResult(pub Option<VictoryState>);

/// Contagion graph topology
#[derive(Resource, Debug, Clone)]
pub struct ContagionGraph {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub rate: f32,
}

impl ContagionGraph {
    pub fn build_city_topology() -> Self {
        Self {
            edges: vec![
                // Downtown connections
                Edge {
                    from: "downtown".to_string(),
                    to: "industrial".to_string(),
                    rate: 0.3,
                },
                Edge {
                    from: "downtown".to_string(),
                    to: "residential".to_string(),
                    rate: 0.4,
                },
                Edge {
                    from: "downtown".to_string(),
                    to: "harbor".to_string(),
                    rate: 0.2,
                },
                // Industrial connections
                Edge {
                    from: "industrial".to_string(),
                    to: "downtown".to_string(),
                    rate: 0.3,
                },
                Edge {
                    from: "industrial".to_string(),
                    to: "residential".to_string(),
                    rate: 0.2,
                },
                // Residential connections
                Edge {
                    from: "residential".to_string(),
                    to: "downtown".to_string(),
                    rate: 0.3,
                },
                Edge {
                    from: "residential".to_string(),
                    to: "suburbs".to_string(),
                    rate: 0.5,
                },
                // Suburbs connections
                Edge {
                    from: "suburbs".to_string(),
                    to: "residential".to_string(),
                    rate: 0.4,
                },
                // Harbor connections
                Edge {
                    from: "harbor".to_string(),
                    to: "downtown".to_string(),
                    rate: 0.3,
                },
                Edge {
                    from: "harbor".to_string(),
                    to: "industrial".to_string(),
                    rate: 0.2,
                },
            ],
        }
    }
}

/// UI state for rendering
#[derive(Resource, Debug, Clone)]
pub struct UIState {
    pub messages: Vec<String>,
    pub selected_district: usize,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            selected_district: 0,
        }
    }
}

impl UIState {
    pub fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }
}
