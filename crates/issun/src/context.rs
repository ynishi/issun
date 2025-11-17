//! Game context for ISSUN
//!
//! GameContext holds persistent data that survives scene transitions.

use crate::resources::Resources;
use crate::service::Service;
use std::any::Any;
use std::collections::HashMap;

/// Marker trait for game context
///
/// Game context should contain only data that:
/// - Persists across scene transitions
/// - Should be saved/loaded
/// - Is shared between scenes
pub trait GameContext: Send + Sync {
    // Marker trait - no required methods
}

/// Default context implementation
///
/// Provides a simple key-value store for game data, service registry, and resources
pub struct Context {
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
    services: HashMap<&'static str, Box<dyn Service>>,
    resources: Resources,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            services: HashMap::new(),
            resources: Resources::new(),
        }
    }

    /// Insert a value into the context
    pub fn insert<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.data.insert(key.into(), Box::new(value));
    }

    /// Get a reference to a value from the context
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key)?.downcast_ref()
    }

    /// Get a mutable reference to a value from the context
    pub fn get_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.data.get_mut(key)?.downcast_mut()
    }

    /// Remove a value from the context
    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// Check if a key exists in the context
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Register a service with the context
    ///
    /// Services are accessible by name and provide reusable functionality.
    /// Typically registered during game initialization via GameBuilder.
    pub fn register_service(&mut self, service: Box<dyn Service>) {
        let name = service.name(); // Returns &'static str
        self.services.insert(name, service);
    }

    /// Access a service by name
    ///
    /// Services are typically stateless or have minimal state.
    /// This method provides immutable access to the service.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get reference to a service
    /// if let Some(combat_service) = ctx.service("combat_service") {
    ///     combat_service.apply_attack(...);
    /// }
    /// ```
    pub fn service(&self, name: &str) -> Option<&dyn Service> {
        self.services.get(name).map(|s| s.as_ref())
    }

    /// Access a service by name with mutable reference
    ///
    /// Use this when the service needs to modify its internal state.
    /// Most services should be stateless and use `service()` instead.
    pub fn service_mut(&mut self, name: &str) -> Option<&mut dyn Service> {
        self.services.get_mut(name).map(|s| s.as_mut())
    }

    /// Get a service with a specific type
    ///
    /// This provides type-safe access via downcasting.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::CombatService;
    ///
    /// if let Some(combat) = ctx.service_as::<CombatService>("combat_service") {
    ///     combat.apply_attack(...);
    /// }
    /// ```
    pub fn service_as<T: Service + 'static>(&self, name: &str) -> Option<&T> {
        self.service(name)?.as_any().downcast_ref::<T>()
    }

    /// Get a service with a specific type (mutable)
    pub fn service_as_mut<T: Service + 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.service_mut(name)?.as_any_mut().downcast_mut::<T>()
    }

    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Get all registered service names
    pub fn service_names(&self) -> Vec<&'static str> {
        self.services.keys().copied().collect()
    }

    /// Get immutable reference to Resources
    ///
    /// Resources contain read-only data like asset databases and configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Access resource
    /// if let Some(enemy_db) = ctx.resources().get::<EnemyDatabase>() {
    ///     println!("Loaded {} enemies", enemy_db.enemies.len());
    /// }
    /// ```
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// Get mutable reference to Resources
    ///
    /// Use this to register resources during game initialization.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Register resource
    /// ctx.resources_mut().insert(EnemyDatabase::from_file("enemies.ron"));
    /// ```
    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

impl GameContext for Context {}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock service for testing
    struct MockCombatService {
        damage_multiplier: f32,
    }

    impl MockCombatService {
        fn new() -> Self {
            Self {
                damage_multiplier: 1.0,
            }
        }

        fn calculate(&self, base: i32) -> i32 {
            (base as f32 * self.damage_multiplier) as i32
        }
    }

    #[async_trait]
    impl Service for MockCombatService {
        fn name(&self) -> &'static str {
            "mock_combat"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_service_registration() {
        let mut ctx = Context::new();
        assert_eq!(ctx.service_count(), 0);

        let service = Box::new(MockCombatService::new());
        ctx.register_service(service);

        assert_eq!(ctx.service_count(), 1);
        assert_eq!(ctx.service_names(), vec!["mock_combat"]);
    }

    #[test]
    fn test_service_access() {
        let mut ctx = Context::new();
        ctx.register_service(Box::new(MockCombatService::new()));

        // Access by name
        let service = ctx.service("mock_combat");
        assert!(service.is_some());
        assert_eq!(service.unwrap().name(), "mock_combat");

        // Access missing service
        let missing = ctx.service("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_service_type_safe_access() {
        let mut ctx = Context::new();
        ctx.register_service(Box::new(MockCombatService::new()));

        // Type-safe access
        let service = ctx.service_as::<MockCombatService>("mock_combat");
        assert!(service.is_some());

        // Use the service
        let damage = service.unwrap().calculate(100);
        assert_eq!(damage, 100);
    }

    #[test]
    fn test_service_mutable_access() {
        let mut ctx = Context::new();
        ctx.register_service(Box::new(MockCombatService::new()));

        // Mutable access
        let service = ctx.service_as_mut::<MockCombatService>("mock_combat");
        assert!(service.is_some());

        // Modify service state
        let service = service.unwrap();
        service.damage_multiplier = 2.0;

        // Verify change
        let service = ctx.service_as::<MockCombatService>("mock_combat").unwrap();
        let damage = service.calculate(100);
        assert_eq!(damage, 200);
    }

    #[test]
    fn test_multiple_services() {
        let mut ctx = Context::new();

        struct ServiceA;
        struct ServiceB;

        #[async_trait]
        impl Service for ServiceA {
            fn name(&self) -> &'static str {
                "service_a"
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }

        #[async_trait]
        impl Service for ServiceB {
            fn name(&self) -> &'static str {
                "service_b"
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }

        ctx.register_service(Box::new(ServiceA));
        ctx.register_service(Box::new(ServiceB));

        assert_eq!(ctx.service_count(), 2);
        assert!(ctx.service("service_a").is_some());
        assert!(ctx.service("service_b").is_some());
    }
}
