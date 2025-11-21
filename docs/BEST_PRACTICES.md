# ISSUN Best Practices

Deep dive into patterns, anti-patterns, and best practices for building games with ISSUN.

---

## 🏗️ Plugin Development Best Practices

### When to Use Derive Macro vs Manual Implementation

**Use `#[derive(Plugin)]` (Recommended)**:
- ✅ Simple plugins with no dependencies
- ✅ Standard resource/state/service/system registration
- ✅ No async initialization needed
- ✅ Cleaner, more maintainable code

**Use Manual Implementation**:
- ✅ Need `dependencies()` to depend on other plugins
- ✅ Need async `initialize()` for file I/O, network setup
- ✅ Complex registration logic
- ✅ Dynamic configuration

**Example - Derive Macro**:
```rust
#[derive(Plugin)]
#[plugin(name = "issun:my_plugin")]
pub struct MyPlugin {
    #[plugin(skip)]
    hook: Arc<dyn MyHook>,
    #[plugin(resource)]
    config: MyConfig,
    #[plugin(runtime_state)]
    state: MyState,
    #[plugin(service)]
    service: MyService,
    #[plugin(system)]
    system: MySystem,
}

impl MyPlugin {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultMyHook);
        Self {
            hook: hook.clone(),
            config: MyConfig::default(),
            state: MyState::new(),
            service: MyService,
            system: MySystem::new(hook),
        }
    }

    pub fn with_hook(mut self, hook: impl MyHook + 'static) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = MySystem::new(hook);
        self
    }
}
```

**Example - Manual Implementation with Dependencies**:
```rust
pub struct AccountingPlugin;

#[async_trait]
impl Plugin for AccountingPlugin {
    fn name(&self) -> &'static str {
        "issun:accounting"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["issun:time"]  // Requires TimePlugin
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_system(Box::new(AccountingSystem::new()));
        builder.register_service(Box::new(AccountingService));
    }
}
```

---

## 🎯 Hook Pattern Best Practices

### Hook Design Principles

**Hooks are for game-specific customization**, not core logic:

✅ **Good Hook Uses**:
- Validate prerequisites before operations
- Calculate dynamic costs based on game state
- Log events to game log
- Update UI state
- Trigger achievements
- Modify effect values based on context

❌ **Bad Hook Uses**:
- Core business logic (belongs in Service/System)
- Direct UI rendering (belongs in Scene)
- State mutations outside ResourceContext
- Long-running operations

### Hook Pattern Example

```rust
// Define the hook trait
#[async_trait]
pub trait PolicyHook: Send + Sync {
    async fn on_policy_activated(
        &self,
        policy: &Policy,
        previous: Option<&Policy>,
        resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    async fn calculate_effect(
        &self,
        policy: &Policy,
        effect_name: &str,
        base_value: f32,
        resources: &ResourceContext,
    ) -> f32 {
        // Default: return base value
        base_value
    }
}

// Default implementation (no-op)
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPolicyHook;

#[async_trait]
impl PolicyHook for DefaultPolicyHook {}

// Game-specific implementation
struct GamePolicyHook;

#[async_trait]
impl PolicyHook for GamePolicyHook {
    async fn calculate_effect(
        &self,
        policy: &Policy,
        effect_name: &str,
        base_value: f32,
        resources: &ResourceContext,
    ) -> f32 {
        // Example: Reduce economic bonuses during winter
        if effect_name.starts_with("income") {
            if let Some(season) = resources.get::<Season>().await {
                if season.is_winter() {
                    return base_value * 0.8;
                }
            }
        }
        base_value
    }
}
```

---

## 📦 Resource vs Runtime State

**Critical Design Principle**: Don't mix read-only definitions with mutable state.

### Resource (Read-Only)

Use `#[plugin(resource)]` for:
- ✅ Asset definitions (Policies, Territories, Factions)
- ✅ Configuration (PolicyConfig, ResearchConfig)
- ✅ Static lookup tables
- ✅ Built-in game data

**Example**:
```rust
#[derive(Plugin)]
#[plugin(name = "issun:policy")]
pub struct PolicyPlugin {
    #[plugin(resource)]
    config: PolicyConfig,        // Read-only configuration
    #[plugin(resource)]
    policies: Policies,           // Read-only policy definitions
    #[plugin(runtime_state)]
    state: PolicyState,           // Mutable runtime state
}
```

### Runtime State (Mutable)

Use `#[plugin(runtime_state)]` for:
- ✅ Active policy IDs
- ✅ Current turn/day
- ✅ Player progress
- ✅ Dynamic game state

**Example**:
```rust
pub type PolicyState = Store<String, PolicyStateEntry>;

#[derive(Clone)]
pub struct PolicyStateEntry {
    pub active_policy_ids: Vec<PolicyId>,  // Changes at runtime
}
```

### Anti-Pattern: Mixing Definitions and State

❌ **Don't Do This**:
```rust
// BAD: Registry mixes everything
pub struct PolicyRegistry {
    policies: HashMap<PolicyId, Policy>,  // Definitions (should be Resource)
    active_policy_id: Option<PolicyId>,   // State (should be Runtime State)
    config: PolicyConfig,                 // Config (should be Resource)
}
```

✅ **Do This Instead**:
```rust
// GOOD: Separated concerns
#[plugin(resource)]
policies: Policies,           // Read-only definitions

#[plugin(resource)]
config: PolicyConfig,         // Read-only configuration

#[plugin(runtime_state)]
state: PolicyState,           // Mutable runtime state
```

---

## 🎮 Service vs System Design

### Service (Pure Logic)

**Characteristics**:
- Stateless (or minimal immutable state)
- Pure functions
- No side effects
- Easy to test

**Examples**:
```rust
#[derive(Service)]
#[service(name = "combat_service")]
pub struct CombatService {
    min_damage: i32,  // Immutable configuration
}

impl CombatService {
    // Pure function: same input → same output
    pub fn calculate_damage(&self, base: i32, defense: i32) -> i32 {
        (base - defense).max(self.min_damage)
    }
}
```

### System (Orchestration)

**Characteristics**:
- Stateful (turn count, logs, etc.)
- Orchestrates multiple services
- Manages game flow
- Mutates ResourceContext

**Examples**:
```rust
#[derive(System)]
#[system(name = "combat_system")]
pub struct CombatSystem {
    turn_count: u32,           // Mutable state
    log: Vec<String>,          // Mutable state
    combat_service: CombatService,  // Uses service
}

impl CombatSystem {
    pub fn process_turn(&mut self, resources: &mut ResourceContext) {
        self.turn_count += 1;

        // Use service for calculations
        let damage = self.combat_service.calculate_damage(10, 5);

        // Mutate resources
        if let Some(mut player) = resources.get_mut::<Player>().await {
            player.hp -= damage;
        }

        // Track state
        self.log.push(format!("Turn {}: {} damage", self.turn_count, damage));
    }
}
```

### When to Use What

**Use Service when**:
- ✅ Logic is purely computational
- ✅ No state needs to be maintained
- ✅ Same logic used in multiple contexts
- ✅ Easy to unit test in isolation

**Use System when**:
- ✅ State must be tracked (turn count, logs)
- ✅ Orchestrating multiple services
- ✅ Managing game flow or loops
- ✅ Need to mutate ResourceContext

---

## 🔄 Event Bus vs Hook

### When to Use Hooks

**Hooks are synchronous and part of the transaction**:
- ✅ Validation (prevent invalid operations)
- ✅ Dynamic calculations (effect values based on game state)
- ✅ Direct resource mutation (within transaction)
- ✅ Performance critical paths

### When to Use Events

**Events are asynchronous and fire-and-forget**:
- ✅ Notifications (UI updates)
- ✅ Audit log / replay
- ✅ Network replication (multiplayer)
- ✅ Loose coupling between systems

### Example: Both Together

```rust
// System uses hook for validation (synchronous)
pub async fn activate_policy(
    policy_id: PolicyId,
    hook: &dyn PolicyHook,
    resources: &mut ResourceContext,
) -> Result<(), String> {
    let policy = get_policy(&policy_id)?;

    // Hook validates (synchronous, part of transaction)
    hook.validate_activation(&policy, resources).await?;

    // Mutate state
    let mut state = resources.get_mut::<PolicyState>().await?;
    state.activate(&policy_id)?;

    // Emit event (asynchronous, fire-and-forget)
    let event_bus = resources.get_mut::<EventBus>().await?;
    event_bus.publish(PolicyActivatedEvent {
        policy_id: policy_id.clone(),
        policy_name: policy.name.clone(),
    });

    Ok(())
}
```

---

## 🧪 Testing Best Practices

### Unit Test Services (Easy)

Services are pure functions, so they're easy to test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_calculation() {
        let service = CombatService { min_damage: 1 };

        assert_eq!(service.calculate_damage(10, 5), 5);
        assert_eq!(service.calculate_damage(10, 15), 1);  // Min damage
    }
}
```

### Integration Test Systems

Systems need ResourceContext, so use builders:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_combat_system() {
        let mut resources = ResourceContext::new();
        resources.register(Player { hp: 100, attack: 10 });

        let mut system = CombatSystem::new();
        system.process_turn(&mut resources).await;

        let player = resources.get::<Player>().await.unwrap();
        assert!(player.hp < 100);
    }
}
```

### Test Hooks with Mocks

```rust
struct MockHook {
    validation_called: Arc<Mutex<bool>>,
}

#[async_trait]
impl PolicyHook for MockHook {
    async fn validate_activation(
        &self,
        _policy: &Policy,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        *self.validation_called.lock().unwrap() = true;
        Ok(())
    }
}

#[tokio::test]
async fn test_hook_called() {
    let called = Arc::new(Mutex::new(false));
    let hook = MockHook { validation_called: called.clone() };

    // ... test logic ...

    assert!(*called.lock().unwrap());
}
```

---

## 📁 Project Structure Best Practices

### Recommended Layout

```
your-game/
├── src/
│   ├── services/           # Pure logic
│   │   ├── mod.rs
│   │   ├── combat.rs
│   │   └── economy.rs
│   │
│   ├── systems/            # Orchestration
│   │   ├── mod.rs
│   │   ├── turn_manager.rs
│   │   └── quest_manager.rs
│   │
│   ├── models/
│   │   ├── resources.rs    # Game entities
│   │   └── scenes.rs       # Scene data
│   │
│   ├── plugins/            # Game plugins
│   │   ├── mod.rs
│   │   └── game_plugin.rs
│   │
│   ├── hooks/              # Hook implementations
│   │   ├── mod.rs
│   │   └── policy_hook.rs
│   │
│   ├── ui/                 # UI code (ratatui)
│   │   ├── mod.rs
│   │   ├── title.rs
│   │   └── game.rs
│   │
│   └── main.rs
│
├── tests/                  # Integration tests
│   └── integration_test.rs
│
└── Cargo.toml
```

---

## 🚫 Common Anti-Patterns

### 1. Game Logic in Scenes

❌ **Don't**:
```rust
impl TitleScene {
    fn handle_input(&mut self, input: InputEvent) {
        if input == InputEvent::Enter {
            // BAD: Damage calculation in UI
            let damage = (self.player.attack - self.enemy.defense).max(1);
            self.enemy.hp -= damage;
        }
    }
}
```

✅ **Do**:
```rust
impl TitleScene {
    fn handle_input(&mut self, input: InputEvent, services: &ServiceContext) {
        if input == InputEvent::Enter {
            // GOOD: Use service
            let combat_service = services.get::<CombatService>();
            let damage = combat_service.calculate_damage(
                self.player.attack,
                self.enemy.defense
            );
            self.enemy.hp -= damage;
        }
    }
}
```

### 2. Stateful Services

❌ **Don't**:
```rust
pub struct CombatService {
    turn_count: u32,  // BAD: Mutable state in service
}

impl CombatService {
    pub fn next_turn(&mut self) {  // BAD: &mut self
        self.turn_count += 1;
    }
}
```

✅ **Do**:
```rust
// Service: pure logic only
pub struct CombatService;

impl CombatService {
    pub fn calculate_damage(&self, base: i32, defense: i32) -> i32 {
        (base - defense).max(1)
    }
}

// System: holds state
pub struct CombatSystem {
    turn_count: u32,  // GOOD: State in system
    combat_service: CombatService,
}
```

### 3. Mixed Responsibilities in Plugin Fields

❌ **Don't**:
```rust
#[derive(Plugin)]
#[plugin(name = "bad")]
pub struct BadPlugin {
    #[plugin(resource)]
    registry: PolicyRegistry,  // BAD: Mixes definitions + state + config
}
```

✅ **Do**:
```rust
#[derive(Plugin)]
#[plugin(name = "good")]
pub struct GoodPlugin {
    #[plugin(resource)]
    policies: Policies,        // Read-only definitions
    #[plugin(resource)]
    config: PolicyConfig,      // Read-only config
    #[plugin(runtime_state)]
    state: PolicyState,        // Mutable state
}
```

---

## 🎯 Performance Tips

### 1. Minimize Runtime State

Keep `#[plugin(runtime_state)]` as small as possible:

✅ **Good**: Store only IDs
```rust
pub struct PolicyState {
    active_policy_ids: Vec<PolicyId>,  // Just IDs
}
```

❌ **Bad**: Store full objects
```rust
pub struct PolicyState {
    active_policies: Vec<Policy>,  // Duplicates definitions
}
```

### 2. Use Services for Hot Paths

Services are faster than Systems for pure calculations:

```rust
// HOT PATH: Use service (no state, inline-friendly)
let damage = combat_service.calculate_damage(base, defense);

// COLD PATH: Use system (stateful, complex logic)
combat_system.process_turn(&mut resources).await;
```

### 3. Batch Event Publishing

Batch events when possible:

```rust
// GOOD: Batch publish
let mut events = vec![];
for policy in policies {
    events.push(PolicyActivatedEvent { ... });
}
event_bus.publish_batch(events);

// LESS EFFICIENT: Individual publishes
for policy in policies {
    event_bus.publish(PolicyActivatedEvent { ... });
}
```

---

## 📖 See Also

- `AGENT.md` - VibeCoding philosophy
- `docs/PLUGIN_LIST.md` - All built-in plugins
- `docs/ARCHITECTURE.md` - Architecture deep dive
- `examples/` - Complete game examples
