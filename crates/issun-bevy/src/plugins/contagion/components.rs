//! Component types for contagion propagation

use bevy::prelude::*;

// ==================== Graph Components ====================

/// Node in the propagation graph (e.g., City, Location)
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct ContagionNode {
    pub node_id: String,
    pub node_type: NodeType,
    pub population: usize,
    pub resistance: f32,
}

impl ContagionNode {
    pub fn new(node_id: impl Into<String>, node_type: NodeType, population: usize) -> Self {
        Self {
            node_id: node_id.into(),
            node_type,
            population,
            resistance: 0.0,
        }
    }

    pub fn with_resistance(mut self, resistance: f32) -> Self {
        self.resistance = resistance.clamp(0.0, 1.0);
        self
    }
}

/// Edge connecting two nodes (e.g., Trade Route)
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct PropagationEdge {
    pub edge_id: String,
    pub from_node: Entity,
    pub to_node: Entity,
    pub transmission_rate: f32,
    pub noise_level: f32,
}

impl PropagationEdge {
    pub fn new(
        edge_id: impl Into<String>,
        from_node: Entity,
        to_node: Entity,
        transmission_rate: f32,
    ) -> Self {
        Self {
            edge_id: edge_id.into(),
            from_node,
            to_node,
            transmission_rate: transmission_rate.clamp(0.0, 1.0),
            noise_level: 0.0,
        }
    }

    pub fn with_noise(mut self, noise: f32) -> Self {
        self.noise_level = noise.clamp(0.0, 1.0);
        self
    }
}

/// Node type classification
#[derive(Clone, Reflect, PartialEq, Debug)]
pub enum NodeType {
    City,
    Village,
    TradingPost,
    MilitaryBase,
    Custom(String),
}

// ==================== Contagion Components ====================

/// Active contagion definition
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct Contagion {
    pub contagion_id: String,
    pub content: ContagionContent,
    pub mutation_rate: f32,
    pub credibility: f32,
    pub origin_node: Entity,
    pub created_at: ContagionDuration,
    pub incubation_duration: ContagionDuration,
    pub active_duration: ContagionDuration,
    pub immunity_duration: ContagionDuration,
    pub reinfection_enabled: bool,
}

/// Infection state at a specific node
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct ContagionInfection {
    pub contagion_entity: Entity,
    pub node_entity: Entity,
    pub state: InfectionState,
    pub infected_at: ContagionDuration,
}

/// Infection state machine
#[derive(Clone, Reflect, PartialEq, Debug)]
pub enum InfectionState {
    Incubating {
        elapsed: ContagionDuration,
        total_duration: ContagionDuration,
    },
    Active {
        elapsed: ContagionDuration,
        total_duration: ContagionDuration,
    },
    Recovered {
        elapsed: ContagionDuration,
        immunity_duration: ContagionDuration,
    },
    Plain,
}

impl InfectionState {
    pub fn get_type(&self) -> super::events::InfectionStateType {
        use super::events::InfectionStateType;
        match self {
            InfectionState::Incubating { .. } => InfectionStateType::Incubating,
            InfectionState::Active { .. } => InfectionStateType::Active,
            InfectionState::Recovered { .. } => InfectionStateType::Recovered,
            InfectionState::Plain => InfectionStateType::Plain,
        }
    }
}

/// Time duration abstraction (Turn/Tick/Time)
#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum ContagionDuration {
    Turns(u64),
    Ticks(u64),
    Seconds(f32),
}

impl ContagionDuration {
    pub fn zero(mode: &super::resources::TimeMode) -> Self {
        use super::resources::TimeMode;
        match mode {
            TimeMode::TurnBased => ContagionDuration::Turns(0),
            TimeMode::TickBased => ContagionDuration::Ticks(0),
            TimeMode::TimeBased => ContagionDuration::Seconds(0.0),
        }
    }

    pub fn is_expired(&self, elapsed: &ContagionDuration) -> bool {
        match (self, elapsed) {
            (ContagionDuration::Turns(total), ContagionDuration::Turns(e)) => e >= total,
            (ContagionDuration::Ticks(total), ContagionDuration::Ticks(e)) => e >= total,
            (ContagionDuration::Seconds(total), ContagionDuration::Seconds(e)) => e >= total,
            _ => false,
        }
    }

    pub fn add(&mut self, delta: &ContagionDuration) {
        match (self, delta) {
            (ContagionDuration::Turns(ref mut e), ContagionDuration::Turns(d)) => *e += d,
            (ContagionDuration::Ticks(ref mut e), ContagionDuration::Ticks(d)) => *e += d,
            (ContagionDuration::Seconds(ref mut e), ContagionDuration::Seconds(d)) => *e += d,
            _ => {}
        }
    }
}

/// Contagion content types
#[derive(Clone, Reflect, Debug)]
pub enum ContagionContent {
    Disease {
        severity: DiseaseLevel,
        location: String,
    },
    ProductReputation {
        product: String,
        sentiment: f32,
    },
    Political {
        faction: String,
        claim: String,
    },
    MarketTrend {
        commodity: String,
        direction: TrendDirection,
    },
    Custom {
        key: String,
        data: String,
    },
}

/// Disease severity levels
#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum DiseaseLevel {
    Mild,
    Moderate,
    Severe,
    Critical,
}

impl DiseaseLevel {
    pub fn increase(self) -> Self {
        match self {
            DiseaseLevel::Mild => DiseaseLevel::Moderate,
            DiseaseLevel::Moderate => DiseaseLevel::Severe,
            DiseaseLevel::Severe => DiseaseLevel::Critical,
            DiseaseLevel::Critical => DiseaseLevel::Critical,
        }
    }

    pub fn decrease(self) -> Self {
        match self {
            DiseaseLevel::Critical => DiseaseLevel::Severe,
            DiseaseLevel::Severe => DiseaseLevel::Moderate,
            DiseaseLevel::Moderate => DiseaseLevel::Mild,
            DiseaseLevel::Mild => DiseaseLevel::Mild,
        }
    }
}

/// Market trend direction
#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}
