//! Test helpers for ISSUN Bevy plugins

use bevy::prelude::*;

/// Test application wrapper for easy plugin testing
pub struct TestApp {
    app: App,
}

impl TestApp {
    /// Create a new test app with minimal plugins
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        Self { app }
    }

    /// Add a plugin to the test app
    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        self.app.add_plugins(plugin);
        self
    }

    /// Update the app (run one frame)
    pub fn update(&mut self) {
        self.app.update();
    }

    /// Get immutable access to the world
    pub fn world(&self) -> &World {
        self.app.world()
    }

    /// Get mutable access to the world
    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }

    /// Spawn an entity with a bundle
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.app.world_mut().spawn(bundle).id()
    }

    /// Send a message (event)
    pub fn send_event<E: Message>(&mut self, event: E) {
        self.app.world_mut().write_message(event);
    }

    /// Read all messages (events) of a type and return them as a Vec
    pub fn read_events<E: Message + Clone>(&mut self) -> Vec<E> {
        let mut events = self.app.world_mut().resource_mut::<Messages<E>>();
        events.drain().collect()
    }

    /// Assert that a message (event) was emitted
    pub fn assert_event_emitted<E: Message>(&self) {
        let events = self.app.world().resource::<Messages<E>>();
        assert!(
            !events.is_empty(),
            "Message {} was not emitted",
            std::any::type_name::<E>()
        );
    }

    /// Assert a component matches a condition
    pub fn assert_component<C: Component>(&self, entity: Entity, validator: impl Fn(&C) -> bool) {
        let component = self.app.world().get::<C>(entity).unwrap_or_else(|| {
            panic!(
                "Component {} not found on entity {:?}",
                std::any::type_name::<C>(),
                entity
            )
        });
        assert!(
            validator(component),
            "Component {} validation failed",
            std::any::type_name::<C>()
        );
    }

    /// Update the app N times
    pub fn update_n_times(&mut self, n: u32) {
        for _ in 0..n {
            self.app.update();
        }
    }
}

impl Default for TestApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct Health {
        current: i32,
        max: i32,
    }

    #[test]
    fn test_app_creation() {
        let app = TestApp::new();
        assert!(app.world().entities().is_empty());
    }

    #[test]
    fn test_spawn_entity() {
        let mut app = TestApp::new();
        let entity = app.spawn(Health {
            current: 100,
            max: 100,
        });

        let health = app.world().get::<Health>(entity).unwrap();
        assert_eq!(health.current, 100);
        assert_eq!(health.max, 100);
    }

    #[test]
    fn test_assert_component() {
        let mut app = TestApp::new();
        let entity = app.spawn(Health {
            current: 50,
            max: 100,
        });

        app.assert_component(entity, |h: &Health| h.current == 50);
        app.assert_component(entity, |h: &Health| h.max == 100);
    }

    #[test]
    fn test_update() {
        let mut app = TestApp::new();
        app.update();
        app.update_n_times(5);
    }
}
