//! Game context for ISSUN
//!
//! GameContext holds persistent data that survives scene transitions.
//!
//! # New Architecture (Proposal C)
//!
//! This module provides three specialized contexts:
//! - `ResourceContext`: Global shared state (Resources)
//! - `ServiceContext`: Stateless domain logic (Services)
//! - `SystemContext`: Stateful orchestration (Systems)

use crate::resources::Resources;
use crate::service::Service;
use crate::state::States;
use crate::system::System;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

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
/// Provides a simple key-value store for game data, service registry, resources, and states
pub struct Context {
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
    services: HashMap<&'static str, Box<dyn Service>>,
    resources: Resources,
    states: States,
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
            states: States::new(),
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

    /// Get immutable reference to States
    ///
    /// States contain mutable runtime game state (save/load targets).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Access state (read-only)
    /// if let Some(territory_state) = ctx.states().get::<TerritoryState>() {
    ///     println!("Control: {:?}", territory_state.control);
    /// }
    /// ```
    pub fn states(&self) -> &States {
        &self.states
    }

    /// Get mutable reference to States
    ///
    /// Use this to access and modify game state during runtime.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Modify state
    /// if let Some(state) = ctx.states_mut().get_mut::<TerritoryState>() {
    ///     state.control.insert(territory_id, 0.5);
    /// }
    /// ```
    pub fn states_mut(&mut self) -> &mut States {
        &mut self.states
    }
}

impl GameContext for Context {}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock service for testing
    #[derive(Clone)]
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

        fn clone_box(&self) -> Box<dyn Service> {
            Box::new(self.clone())
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

        #[derive(Clone)]
        struct ServiceA;
        #[derive(Clone)]
        struct ServiceB;

        #[async_trait]
        impl Service for ServiceA {
            fn name(&self) -> &'static str {
                "service_a"
            }

            fn clone_box(&self) -> Box<dyn Service> {
                Box::new(self.clone())
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

            fn clone_box(&self) -> Box<dyn Service> {
                Box::new(self.clone())
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

// ==================== New Architecture (Proposal C) ====================

/// Shared resource container type (`Arc<RwLock<...>>`)
type Resource = Arc<RwLock<Box<dyn Any + Send + Sync>>>;

/// Read guard for a resource in ResourceContext
///
/// This guard dereferences to &T and releases the lock on drop.
pub struct ResourceReadGuard<T: 'static> {
    guard: OwnedRwLockReadGuard<Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<T>,
}

impl<T: 'static> Deref for ResourceReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard
            .downcast_ref::<T>()
            .expect("Resource type mismatch - this is a bug")
    }
}

/// Write guard for a resource in ResourceContext
///
/// This guard dereferences to &mut T and releases the lock on drop.
pub struct ResourceWriteGuard<T: 'static> {
    guard: OwnedRwLockWriteGuard<Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<T>,
}

impl<T: 'static> Deref for ResourceWriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard
            .downcast_ref::<T>()
            .expect("Resource type mismatch - this is a bug")
    }
}

impl<T: 'static> DerefMut for ResourceWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .downcast_mut::<T>()
            .expect("Resource type mismatch - this is a bug")
    }
}

/// Container for global, shared state (Resources)
///
/// Thread-safe with async RwLock for concurrent access:
/// - Multiple readers OR single writer
/// - Immutable during scene rendering, mutable in systems
///
/// # Example
///
/// ```ignore
/// let mut resources = ResourceContext::new();
/// resources.insert(Player::new("Hero"));
/// resources.insert(Score(0));
///
/// // Read access (multiple readers allowed)
/// let player = resources.get::<Player>().await.unwrap();
/// println!("Player HP: {}", player.hp);
///
/// // Write access (exclusive)
/// let mut player = resources.get_mut::<Player>().await.unwrap();
/// player.hp -= 10;
/// ```
pub struct ResourceContext {
    resources: HashMap<TypeId, Resource>,
}

impl ResourceContext {
    /// Create a new empty ResourceContext
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Insert a resource into the context
    ///
    /// # Example
    ///
    /// ```ignore
    /// resources.insert(Player::new("Hero"));
    /// resources.insert(Inventory::default());
    /// ```
    pub fn insert<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources
            .insert(TypeId::of::<T>(), Arc::new(RwLock::new(Box::new(resource))));
    }

    /// Insert a pre-boxed resource into the context (internal use)
    ///
    /// Used by GameBuilder to transfer resources from plugin registration.
    pub(crate) fn insert_boxed(&mut self, type_id: TypeId, boxed: Box<dyn std::any::Any + Send + Sync>) {
        self.resources
            .insert(type_id, Arc::new(RwLock::new(boxed)));
    }

    /// Get immutable reference to a resource (async read lock)
    ///
    /// Returns a guard that dereferences to &T.
    /// Multiple readers can access simultaneously.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let player = resources.get::<Player>().await?;
    /// println!("HP: {}", player.hp);
    /// // Guard automatically released when dropped
    /// ```
    pub async fn get<T: 'static + Send + Sync>(&self) -> Option<ResourceReadGuard<T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?.clone();
        let guard = resource.read_owned().await;
        Some(ResourceReadGuard {
            guard,
            _marker: PhantomData,
        })
    }

    /// Get mutable reference to a resource (async write lock)
    ///
    /// Returns a guard that dereferences to &mut T.
    /// Exclusive access - blocks all other readers and writers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut player = resources.get_mut::<Player>().await?;
    /// player.hp -= 10;
    /// // Guard automatically released when dropped
    /// ```
    pub async fn get_mut<T: 'static + Send + Sync>(&self) -> Option<ResourceWriteGuard<T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?.clone();
        let guard = resource.write_owned().await;
        Some(ResourceWriteGuard {
            guard,
            _marker: PhantomData,
        })
    }

    /// Try to get immutable reference to a resource without awaiting
    pub fn try_get<T: 'static + Send + Sync>(&self) -> Option<ResourceReadGuard<T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?.clone();
        let guard = resource.try_read_owned().ok()?;
        Some(ResourceReadGuard {
            guard,
            _marker: PhantomData,
        })
    }

    /// Try to get mutable reference to a resource without awaiting
    pub fn try_get_mut<T: 'static + Send + Sync>(&self) -> Option<ResourceWriteGuard<T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?.clone();
        let guard = resource.try_write_owned().ok()?;
        Some(ResourceWriteGuard {
            guard,
            _marker: PhantomData,
        })
    }

    /// Check if a resource exists
    pub fn contains<T: 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// Remove a resource from the context
    pub fn remove<T: 'static>(&mut self) -> bool {
        self.resources.remove(&TypeId::of::<T>()).is_some()
    }

    /// Get the number of registered resources
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

impl Default for ResourceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Container for stateless Services (pure domain logic)
///
/// Services provide pure, reusable functionality without state.
///
/// # Example
///
/// ```ignore
/// let mut services = ServiceContext::new();
/// services.register(Box::new(CombatService::new()));
/// services.register(Box::new(LootService::new()));
///
/// // Access by name
/// let combat = services.get_as::<CombatService>("combat_service")?;
/// let damage = combat.calculate_damage(attack, defense);
/// ```
pub struct ServiceContext {
    services: HashMap<&'static str, Box<dyn Service>>,
}

impl ServiceContext {
    /// Create a new empty ServiceContext
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a service with the context
    ///
    /// Services are identified by their name() method.
    ///
    /// # Example
    ///
    /// ```ignore
    /// services.register(Box::new(CombatService::new()));
    /// ```
    pub fn register(&mut self, service: Box<dyn Service>) {
        let name = service.name();
        self.services.insert(name, service);
    }

    /// Get reference to a service by name
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(service) = services.get("combat_service") {
    ///     println!("Found service: {}", service.name());
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<&dyn Service> {
        self.services.get(name).map(|boxed| boxed.as_ref())
    }

    /// Get mutable reference to a service by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Service> {
        self.services.get_mut(name).map(|boxed| boxed.as_mut())
    }

    /// Get type-safe reference to a service
    ///
    /// # Example
    ///
    /// ```ignore
    /// let combat = services.get_as::<CombatService>("combat_service")?;
    /// let damage = combat.calculate_damage(100, 20);
    /// ```
    pub fn get_as<T: Service + 'static>(&self, name: &str) -> Option<&T> {
        self.get(name)?.as_any().downcast_ref::<T>()
    }

    /// Get type-safe mutable reference to a service
    pub fn get_as_mut<T: Service + 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.get_mut(name)?.as_any_mut().downcast_mut::<T>()
    }

    /// Get the number of registered services
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }

    /// Get all registered service names
    pub fn service_names(&self) -> Vec<&'static str> {
        self.services.keys().copied().collect()
    }
}

impl Default for ServiceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Container for stateful Systems (application logic & orchestration)
///
/// Systems are owned by SceneDirector and passed as &mut, so no Arc needed.
/// Systems orchestrate game logic using Services and manage their own state.
///
/// # Example
///
/// ```ignore
/// let mut systems = SystemContext::new();
/// systems.register(CombatSystem::new());
/// systems.register(TurnManager::new());
///
/// // Access with type-safe API
/// let combat = systems.get_mut::<CombatSystem>()?;
/// combat.process_turn(resources).await?;
/// ```
pub struct SystemContext {
    systems: HashMap<TypeId, Box<dyn System>>,
}

impl SystemContext {
    /// Create a new empty SystemContext
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
        }
    }

    /// Register a system with the context
    ///
    /// Systems are identified by their TypeId.
    ///
    /// # Example
    ///
    /// ```ignore
    /// systems.register(CombatSystem::new());
    /// systems.register(TurnManager::new());
    /// ```
    pub fn register<T: System + 'static>(&mut self, system: T) {
        self.systems.insert(TypeId::of::<T>(), Box::new(system));
    }

    /// Register an already boxed system (used by GameBuilder)
    pub fn register_boxed(&mut self, system: Box<dyn System>) {
        let type_id = system.as_any().type_id();
        self.systems.insert(type_id, system);
    }

    /// Get immutable reference to a system
    ///
    /// # Example
    ///
    /// ```ignore
    /// let combat = systems.get::<CombatSystem>()?;
    /// let turn_count = combat.turn_count();
    /// ```
    pub fn get<T: System + 'static>(&self) -> Option<&T> {
        self.systems
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<T>()
    }

    /// Get mutable reference to a system
    ///
    /// # Example
    ///
    /// ```ignore
    /// let combat = systems.get_mut::<CombatSystem>()?;
    /// combat.execute_player_attack(resources).await?;
    /// ```
    pub fn get_mut<T: System + 'static>(&mut self) -> Option<&mut T> {
        self.systems
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    /// Check if a system exists
    pub fn contains<T: System + 'static>(&self) -> bool {
        self.systems.contains_key(&TypeId::of::<T>())
    }

    /// Remove a system from the context
    pub fn remove<T: System + 'static>(&mut self) -> bool {
        self.systems.remove(&TypeId::of::<T>()).is_some()
    }

    /// Get the number of registered systems
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}

impl Default for SystemContext {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Tests for New Architecture ====================

#[cfg(test)]
mod new_context_tests {
    use super::*;
    use async_trait::async_trait;

    // Test Resource
    #[derive(Debug, Clone, PartialEq)]
    struct Player {
        name: String,
        hp: i32,
        max_hp: i32,
    }

    impl Player {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                hp: 100,
                max_hp: 100,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Score(i32);

    // Test Service
    #[derive(Clone)]
    struct TestCombatService;

    impl TestCombatService {
        fn calculate_damage(&self, attack: i32, defense: i32) -> i32 {
            (attack - defense).max(1)
        }
    }

    #[async_trait]
    impl Service for TestCombatService {
        fn name(&self) -> &'static str {
            "test_combat"
        }

        fn clone_box(&self) -> Box<dyn Service> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    // Test System
    struct TestCombatSystem {
        turn_count: u32,
    }

    impl TestCombatSystem {
        fn new() -> Self {
            Self { turn_count: 0 }
        }

        fn increment_turn(&mut self) {
            self.turn_count += 1;
        }
    }

    #[async_trait]
    impl System for TestCombatSystem {
        fn name(&self) -> &'static str {
            "test_combat_system"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    // ===== ResourceContext Tests =====

    #[tokio::test]
    async fn test_resource_context_insert_and_get() {
        let mut resources = ResourceContext::new();
        resources.insert(Player::new("Hero"));

        let player = resources.get::<Player>().await.unwrap();
        assert_eq!(player.name, "Hero");
        assert_eq!(player.hp, 100);
    }

    #[tokio::test]
    async fn test_resource_context_get_mut() {
        let mut resources = ResourceContext::new();
        resources.insert(Player::new("Hero"));

        {
            let mut player = resources.get_mut::<Player>().await.unwrap();
            player.hp -= 30;
        }

        let player = resources.get::<Player>().await.unwrap();
        assert_eq!(player.hp, 70);
    }

    #[tokio::test]
    async fn test_resource_context_multiple_resources() {
        let mut resources = ResourceContext::new();
        resources.insert(Player::new("Hero"));
        resources.insert(Score(100));

        assert_eq!(resources.len(), 2);

        let player = resources.get::<Player>().await.unwrap();
        let score = resources.get::<Score>().await.unwrap();

        assert_eq!(player.name, "Hero");
        assert_eq!(score.0, 100);
    }

    #[tokio::test]
    async fn test_resource_context_contains() {
        let mut resources = ResourceContext::new();
        resources.insert(Player::new("Hero"));

        assert!(resources.contains::<Player>());
        assert!(!resources.contains::<Score>());
    }

    #[tokio::test]
    async fn test_resource_context_remove() {
        let mut resources = ResourceContext::new();
        resources.insert(Player::new("Hero"));

        assert!(resources.contains::<Player>());
        assert!(resources.remove::<Player>());
        assert!(!resources.contains::<Player>());
    }

    #[tokio::test]
    async fn test_resource_context_concurrent_reads() {
        let mut resources = ResourceContext::new();
        resources.insert(Player::new("Hero"));

        // Multiple concurrent readers should work
        let reader1 = resources.get::<Player>().await.unwrap();
        let reader2 = resources.get::<Player>().await.unwrap();

        assert_eq!(reader1.name, "Hero");
        assert_eq!(reader2.name, "Hero");
    }

    // ===== ServiceContext Tests =====

    #[test]
    fn test_service_context_register_and_get() {
        let mut services = ServiceContext::new();
        services.register(Box::new(TestCombatService));

        let service = services.get("test_combat").unwrap();
        assert_eq!(service.name(), "test_combat");
    }

    #[test]
    fn test_service_context_get_as() {
        let mut services = ServiceContext::new();
        services.register(Box::new(TestCombatService));

        let combat = services.get_as::<TestCombatService>("test_combat").unwrap();
        let damage = combat.calculate_damage(100, 30);
        assert_eq!(damage, 70);
    }

    #[test]
    fn test_service_context_service_names() {
        let mut services = ServiceContext::new();
        services.register(Box::new(TestCombatService));

        let names = services.service_names();
        assert_eq!(names, vec!["test_combat"]);
    }

    #[test]
    fn test_service_context_len_and_is_empty() {
        let mut services = ServiceContext::new();
        assert!(services.is_empty());
        assert_eq!(services.len(), 0);

        services.register(Box::new(TestCombatService));
        assert!(!services.is_empty());
        assert_eq!(services.len(), 1);
    }

    // ===== SystemContext Tests =====

    #[test]
    fn test_system_context_register_and_get() {
        let mut systems = SystemContext::new();
        systems.register(TestCombatSystem::new());

        let system = systems.get::<TestCombatSystem>().unwrap();
        assert_eq!(system.turn_count, 0);
    }

    #[test]
    fn test_system_context_get_mut() {
        let mut systems = SystemContext::new();
        systems.register(TestCombatSystem::new());

        {
            let system = systems.get_mut::<TestCombatSystem>().unwrap();
            system.increment_turn();
            system.increment_turn();
        }

        let system = systems.get::<TestCombatSystem>().unwrap();
        assert_eq!(system.turn_count, 2);
    }

    #[test]
    fn test_system_context_contains() {
        let mut systems = SystemContext::new();
        systems.register(TestCombatSystem::new());

        assert!(systems.contains::<TestCombatSystem>());
        assert_eq!(systems.len(), 1);
    }

    #[test]
    fn test_system_context_remove() {
        let mut systems = SystemContext::new();
        systems.register(TestCombatSystem::new());

        assert!(systems.contains::<TestCombatSystem>());
        assert!(systems.remove::<TestCombatSystem>());
        assert!(!systems.contains::<TestCombatSystem>());
    }

    #[test]
    fn test_system_context_len_and_is_empty() {
        let mut systems = SystemContext::new();
        assert!(systems.is_empty());
        assert_eq!(systems.len(), 0);

        systems.register(TestCombatSystem::new());
        assert!(!systems.is_empty());
        assert_eq!(systems.len(), 1);
    }
}
