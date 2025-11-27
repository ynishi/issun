//! Reflect-compatible wrapper types for ContagionState
//!
//! These wrappers enable Bevy's reflection system for trace/replay functionality.
//! They wrap the generic ContagionState<M> with concrete types that can derive Reflect.

use bevy::prelude::*;
use issun_core::mechanics::contagion::prelude::*;

use super::components::ContagionState;

// ==================== Reflect Wrappers ====================

/// Reflect-compatible wrapper for SimpleVirusState
///
/// Use this instead of ContagionState<SimpleVirus> when you need Bevy reflection
/// support (e.g., for Inspector, tracing, or replay systems).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SimpleVirusStateReflect {
    #[reflect(ignore)]
    inner: ContagionState<SimpleVirus>,
}

impl SimpleVirusStateReflect {
    pub fn new(severity: u32) -> Self {
        Self {
            inner: ContagionState::new(severity),
        }
    }

    pub fn severity(&self) -> u32 {
        self.inner.severity()
    }

    pub fn is_infected(&self) -> bool {
        self.inner.is_infected()
    }

    pub fn state_mut(&mut self) -> &mut SimpleSeverity {
        &mut self.inner.state
    }

    /// Convert to the generic ContagionState for use with game logic
    pub fn into_inner(self) -> ContagionState<SimpleVirus> {
        self.inner
    }

    /// Borrow the inner ContagionState
    pub fn inner(&self) -> &ContagionState<SimpleVirus> {
        &self.inner
    }

    /// Mutably borrow the inner ContagionState
    pub fn inner_mut(&mut self) -> &mut ContagionState<SimpleVirus> {
        &mut self.inner
    }
}

/// Reflect-compatible wrapper for ExplosiveVirusState
///
/// Use this instead of ContagionState<ExplosiveVirus> when you need Bevy reflection
/// support (e.g., for Inspector, tracing, or replay systems).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ExplosiveVirusStateReflect {
    #[reflect(ignore)]
    inner: ContagionState<ExplosiveVirus>,
}

impl ExplosiveVirusStateReflect {
    pub fn new(severity: u32) -> Self {
        Self {
            inner: ContagionState::new(severity),
        }
    }

    pub fn severity(&self) -> u32 {
        self.inner.severity()
    }

    pub fn is_infected(&self) -> bool {
        self.inner.is_infected()
    }

    pub fn state_mut(&mut self) -> &mut SimpleSeverity {
        &mut self.inner.state
    }

    /// Convert to the generic ContagionState for use with game logic
    pub fn into_inner(self) -> ContagionState<ExplosiveVirus> {
        self.inner
    }

    /// Borrow the inner ContagionState
    pub fn inner(&self) -> &ContagionState<ExplosiveVirus> {
        &self.inner
    }

    /// Mutably borrow the inner ContagionState
    pub fn inner_mut(&mut self) -> &mut ContagionState<ExplosiveVirus> {
        &mut self.inner
    }
}

/// Reflect-compatible wrapper for ZombieVirusState
///
/// Use this instead of ContagionState<ZombieVirus> when you need Bevy reflection
/// support (e.g., for Inspector, tracing, or replay systems).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ZombieVirusStateReflect {
    #[reflect(ignore)]
    inner: ContagionState<ZombieVirus>,
}

impl ZombieVirusStateReflect {
    pub fn new(severity: u32) -> Self {
        Self {
            inner: ContagionState::new(severity),
        }
    }

    pub fn severity(&self) -> u32 {
        self.inner.severity()
    }

    pub fn is_infected(&self) -> bool {
        self.inner.is_infected()
    }

    pub fn state_mut(&mut self) -> &mut SimpleSeverity {
        &mut self.inner.state
    }

    /// Convert to the generic ContagionState for use with game logic
    pub fn into_inner(self) -> ContagionState<ZombieVirus> {
        self.inner
    }

    /// Borrow the inner ContagionState
    pub fn inner(&self) -> &ContagionState<ZombieVirus> {
        &self.inner
    }

    /// Mutably borrow the inner ContagionState
    pub fn inner_mut(&mut self) -> &mut ContagionState<ZombieVirus> {
        &mut self.inner
    }
}
