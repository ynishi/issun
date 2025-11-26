//! Entity Lifetime Safety
//!
//! Provides safe wrappers for Entity access from scripts.
//! All entity access must check if the entity still exists before accessing components.

use bevy::prelude::*;

use super::backend::ScriptError;

/// Safe entity reference that checks lifetime
///
/// This wrapper ensures that accessing despawned entities returns an error
/// instead of causing undefined behavior or crashes.
pub struct SafeEntityRef<'w> {
    entity: Entity,
    world: &'w World,
}

impl<'w> SafeEntityRef<'w> {
    /// Create a new safe entity reference
    ///
    /// This does NOT check if the entity exists yet - that happens on access.
    pub fn new(entity: Entity, world: &'w World) -> Self {
        Self { entity, world }
    }

    /// Check if entity still exists
    ///
    /// # P0 Safety Check
    /// This is the mandatory safety check that prevents crashes.
    pub fn exists(&self) -> bool {
        self.world.get_entity(self.entity).is_ok()
    }

    /// Get entity ID
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// Get a component from the entity (safe)
    ///
    /// Returns error if entity has been despawned.
    pub fn get_component<T: Component>(&self) -> Result<Option<&T>, ScriptError> {
        // P0 SAFETY CHECK: Verify entity exists before access
        let entity_ref = self.world.get_entity(self.entity).map_err(|_entity| {
            ScriptError::EntityDespawned(format!(
                "Entity {:?} has been despawned and cannot be accessed",
                self.entity
            ))
        })?;

        Ok(entity_ref.get::<T>())
    }

    /// Check if entity has a component (safe)
    ///
    /// Returns error if entity has been despawned.
    pub fn has_component<T: Component>(&self) -> Result<bool, ScriptError> {
        // P0 SAFETY CHECK: Verify entity exists before access
        let entity_ref = self.world.get_entity(self.entity).map_err(|_entity| {
            ScriptError::EntityDespawned(format!(
                "Entity {:?} has been despawned and cannot be accessed",
                self.entity
            ))
        })?;

        Ok(entity_ref.contains::<T>())
    }
}

/// Helper to convert Entity ID (u64) to safe entity reference
///
/// # Safety Note
/// This function uses Entity::from_bits() which is ALWAYS followed by
/// a safety check in SafeEntityRef methods.
pub fn entity_from_bits_safe(entity_bits: u64, world: &World) -> SafeEntityRef<'_> {
    let entity = Entity::from_bits(entity_bits);
    SafeEntityRef::new(entity, world)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_entity_ref_exists() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let safe_ref = SafeEntityRef::new(entity, &world);
        assert!(safe_ref.exists());
    }

    #[test]
    fn test_safe_entity_ref_despawned() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Despawn entity
        world.despawn(entity);

        let safe_ref = SafeEntityRef::new(entity, &world);
        assert!(!safe_ref.exists());
    }

    #[test]
    fn test_safe_entity_ref_get_component_success() {
        let mut world = World::new();
        let entity = world.spawn(Name::new("Test")).id();

        let safe_ref = SafeEntityRef::new(entity, &world);
        let result = safe_ref.get_component::<Name>();

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_safe_entity_ref_get_component_despawned() {
        let mut world = World::new();
        let entity = world.spawn(Name::new("Test")).id();

        // Despawn entity
        world.despawn(entity);

        let safe_ref = SafeEntityRef::new(entity, &world);
        let result = safe_ref.get_component::<Name>();

        // Should return error, NOT crash
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ScriptError::EntityDespawned(_)
        ));
    }

    #[test]
    fn test_safe_entity_ref_has_component_despawned() {
        let mut world = World::new();
        let entity = world.spawn(Name::new("Test")).id();

        // Despawn entity
        world.despawn(entity);

        let safe_ref = SafeEntityRef::new(entity, &world);
        let result = safe_ref.has_component::<Name>();

        // Should return error, NOT crash
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ScriptError::EntityDespawned(_)
        ));
    }

    #[test]
    fn test_entity_from_bits_safe() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let entity_bits = entity.to_bits();

        let safe_ref = entity_from_bits_safe(entity_bits, &world);
        assert!(safe_ref.exists());
        assert_eq!(safe_ref.id(), entity);
    }
}
