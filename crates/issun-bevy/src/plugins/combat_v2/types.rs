//! Bevy-specific types and adapters for combat_v2 plugin.
//!
//! This module provides the glue between issun-core's pure combat logic
//! and Bevy's ECS system.

use bevy::{ecs::message::MessageWriter, prelude::*};
use issun_core::mechanics::combat::{CombatConfig, CombatEvent, CombatState};
use issun_core::mechanics::EventEmitter;

/// Combat configuration resource - wraps issun-core's CombatConfig
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct CombatConfigResource {
    #[reflect(ignore)]
    pub config: CombatConfig,
}

impl CombatConfigResource {
    pub fn new(config: CombatConfig) -> Self {
        Self { config }
    }
}

/// Message wrapper for issun-core's CombatEvent (Bevy 0.17+)
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct CombatEventWrapper {
    pub entity: Entity,
    pub event: CombatEvent,
}

/// Bevy adapter: Wraps Bevy's MessageWriter to implement EventEmitter.
///
/// This allows issun-core's combat mechanic to emit events into Bevy's
/// message system without depending on Bevy directly.
pub struct BevyEventEmitter<'a, 'b> {
    entity: Entity,
    writer: &'a mut MessageWriter<'b, CombatEventWrapper>,
}

impl<'a, 'b> BevyEventEmitter<'a, 'b> {
    /// Create a new Bevy event emitter.
    pub fn new(entity: Entity, writer: &'a mut MessageWriter<'b, CombatEventWrapper>) -> Self {
        Self { entity, writer }
    }
}

impl<'a, 'b> EventEmitter<CombatEvent> for BevyEventEmitter<'a, 'b> {
    fn emit(&mut self, event: CombatEvent) {
        self.writer.write(CombatEventWrapper {
            entity: self.entity,
            event,
        });
    }
}

/// Component: Health (maps to CombatState internally).
///
/// This component stores the combat state (HP) for an entity.
/// It wraps issun-core's CombatState but provides Bevy-specific functionality.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Health {
    /// Current hit points
    pub current: i32,

    /// Maximum hit points
    pub max: i32,
}

impl Health {
    /// Create a new Health component with the given max HP.
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    /// Convert to issun-core's CombatState.
    pub fn to_combat_state(&self) -> CombatState {
        CombatState {
            current_hp: self.current,
            max_hp: self.max,
        }
    }

    /// Update from issun-core's CombatState.
    pub fn from_combat_state(&mut self, state: &CombatState) {
        self.current = state.current_hp;
        self.max = state.max_hp;
    }

    /// Check if this entity is alive.
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    /// Check if this entity is dead.
    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

/// Component: Attack power.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Attack {
    /// Attack power value
    pub power: i32,
}

/// Component: Defense value.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Defense {
    /// Defense value
    pub value: i32,
}

/// Component: Elemental type (wraps issun-core's Element).
#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component)]
pub struct ElementType {
    /// The element this entity belongs to
    #[reflect(ignore)]
    pub element: issun_core::mechanics::combat::Element,
}

/// Message: Request damage calculation.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct DamageRequested {
    /// Entity that is attacking
    pub attacker: Entity,

    /// Entity that is being attacked
    pub target: Entity,
}

/// Message: Damage was applied to a target.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct DamageApplied {
    /// Entity that attacked
    pub attacker: Entity,

    /// Entity that received damage
    pub target: Entity,

    /// Amount of damage dealt
    pub amount: i32,

    /// Whether this was a critical hit
    pub is_critical: bool,

    /// Whether this damage killed the target
    pub is_fatal: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_component() {
        let health = Health::new(100);
        assert_eq!(health.current, 100);
        assert_eq!(health.max, 100);
        assert!(health.is_alive());
        assert!(!health.is_dead());
    }

    #[test]
    fn test_health_to_combat_state() {
        let health = Health {
            current: 75,
            max: 100,
        };
        let state = health.to_combat_state();
        assert_eq!(state.current_hp, 75);
        assert_eq!(state.max_hp, 100);
    }

    #[test]
    fn test_health_from_combat_state() {
        let mut health = Health::new(100);
        let state = CombatState {
            current_hp: 50,
            max_hp: 100,
        };
        health.from_combat_state(&state);
        assert_eq!(health.current, 50);
        assert_eq!(health.max, 100);
    }
}
