# Whispers of Plague

A **plague simulation game** demonstrating ISSUN's **VibeCoding** philosophy with custom plugin development and strategic gameplay.

## 🎮 Game Concept

Play as either a **Plague** (spread infection) or **Savior** (save the city) across 20 turns. Manipulate public perception through **rumors** while managing virus mutations and district populations.

### Game Modes

- **🦠 Plague Mode**: Spread infection across districts. Win by infecting >50% of the population.
- **💉 Savior Mode**: Save the city from disease. Win by keeping >50% healthy until turn 20.

## 🏗️ Architecture (VibeCoding in Action)

This example demonstrates **ISSUN's 80/20 principle**:
- **80% Framework**: Uses existing ISSUN patterns (Plugin system, Service/System separation, Hook customization)
- **20% Game Logic**: Custom virus mechanics, rumor system, and win conditions

### Plugin Structure

```
whispers-of-plague/
├── src/
│   ├── plugins/
│   │   ├── plague.rs          # PlagueGamePlugin (virus + win condition)
│   │   └── rumor/             # RumorPlugin (complete sub-plugin)
│   │       ├── hook.rs        # Hook for game-specific rumor logic
│   │       ├── models.rs      # Rumor definitions & state
│   │       ├── plugin.rs      # Plugin registration (#[derive(Plugin)])
│   │       ├── service.rs     # Pure rumor calculations
│   │       └── system.rs      # Rumor orchestration
│   ├── services/              # Pure logic services
│   │   ├── virus.rs           # Infection spread calculations
│   │   └── win_condition.rs  # Victory/defeat detection
│   ├── systems/
│   │   └── turn.rs            # Turn management
│   ├── models/
│   │   ├── resources.rs       # Game data (CityMap, Virus, District)
│   │   ├── context.rs         # Game context (turn, mode)
│   │   ├── game_scene.rs      # Scene enum
│   │   └── scenes/            # Scene data structs
│   └── ui/
│       └── mod.rs             # Ratatui rendering
└── Cargo.toml
```

## 🎯 Key Implementations

### 1. Custom Plugin with `#[derive(Plugin)]`

```rust
#[derive(Plugin)]
#[plugin(name = "issun:rumor")]
pub struct RumorPlugin {
    #[plugin(skip)]
    hook: Arc<dyn RumorHook>,
    #[plugin(resource)]
    registry: RumorRegistry,
    #[plugin(runtime_state)]
    state: RumorState,
    #[plugin(service)]
    service: RumorService,
    #[plugin(system)]
    system: RumorSystem,
}
```

**Benefits**:
- Automatic `Plugin` trait implementation
- Clean field annotations (`#[plugin(resource)]`, `#[plugin(system)]`, etc.)
- No boilerplate `build()` method

### 2. Hook Pattern for Customization

```rust
#[async_trait]
pub trait RumorHook: Send + Sync {
    async fn on_before_apply(&self, rumor: &Rumor, resources: &ResourceContext)
        -> Result<(), String>;
    async fn calculate_migration_target(&self, from_idx: usize, resources: &ResourceContext)
        -> Option<usize>;
}
```

**Usage**: Inject game-specific logic (migration targeting, rumor validation) without modifying core plugin.

### 3. Service/System Separation

**VirusService** (Pure Logic):
```rust
impl VirusService {
    pub fn calculate_spread(&self, district: &District, virus: &Virus, panic: f32) -> u32 {
        let base_spread = (district.infected as f32 * virus.spread_rate * (1.0 + panic)).round() as u32;
        base_spread.min(district.healthy())
    }
}
```

**TurnSystem** (Orchestration):
```rust
impl TurnSystem {
    pub async fn process_turn(&mut self, resources: &mut ResourceContext) {
        // 1. Use VirusService for calculations
        // 2. Mutate ResourceContext
        // 3. Check win conditions
    }
}
```

## 🚀 Running the Game

```bash
cd examples/whispers-of-plague
cargo run
```

**Controls**:
- **1/2**: Select Plague/Savior mode (Title screen)
- **N**: Next turn
- **R**: Spread rumor
- **Q**: Quit

## 📊 Game Mechanics

### Virus System
- **Mutations**: Virus mutates at 50k/100k infections
  - Alpha → Beta: +30% spread, +20% lethality
  - Beta → Gamma: +50% spread, +40% lethality
- **Spread Formula**: `infected × spread_rate × (1 + panic_level)`

### Rumor System
- **Credibility Decay**: 90% per turn (configurable)
- **Max Active Rumors**: 3 simultaneous
- **Effects**:
  - `IncreasePanic(delta)` - Raises panic levels
  - `DecreasePanic(delta)` - Lowers panic levels
  - `PromoteMigration {rate}` - Moves population between districts
  - `PromoteIsolation {panic_reduction}` - Reduces panic via isolation

### District Model
```rust
pub struct District {
    pub population: u32,
    pub infected: u32,
    pub dead: u32,
    pub panic_level: f32,  // 0.0-1.0
}
```

## 🎓 Learning Points

### 1. **Plugin Derive Macro**
- Use `#[derive(Plugin)]` for clean plugin structure
- Field annotations: `#[plugin(resource)]`, `#[plugin(system)]`, `#[plugin(service)]`
- `#[plugin(skip)]` for hooks and internal state

### 2. **Service Pattern**
- Pure, stateless logic in `Service`
- Easy to test (no mocking needed)
- Example: `VirusService::calculate_spread()`

### 3. **System Pattern**
- Stateful orchestration in `System`
- Uses `ServiceContext` to call services
- Mutates `ResourceContext` for game state

### 4. **Hook Pattern**
- Inject custom logic without modifying plugins
- Example: `RumorHook::calculate_migration_target()`

### 5. **Scene Architecture**
- **Scene Enum**: `GameScene::Title | Game | Result`
- **Scene Data**: Lightweight view state (selected district, log messages)
- **Input Handling**: Async scene transitions

## 📝 Extending the Game

### Add New Rumor Type

1. **Add to `RumorEffect` enum**:
```rust
pub enum RumorEffect {
    IncreasePanic(f32),
    // ...
    InduceQuarantine { district_id: String },  // NEW
}
```

2. **Implement in `RumorSystem::apply_effect()`**:
```rust
RumorEffect::InduceQuarantine { district_id } => {
    // Lock down district, reduce spread
}
```

3. **Register in `RumorRegistry::new()`**:
```rust
registry.add(Rumor {
    id: "quarantine_order",
    effect: RumorEffect::InduceQuarantine { district_id: "downtown".into() },
    // ...
});
```

### Add New Service

```rust
#[derive(Clone, Default, DeriveService)]
#[service(name = "vaccine_service")]
pub struct VaccineService;

impl VaccineService {
    pub fn calculate_immunization(&self, vaccinated: u32, efficiency: f32) -> u32 {
        // Pure logic here
    }
}
```

### Custom Win Condition Hook

```rust
struct CustomWinHook;

impl WinConditionHook for CustomWinHook {
    async fn check_custom_victory(&self, resources: &ResourceContext) -> Option<VictoryResult> {
        // Custom win logic (e.g., "All districts reach herd immunity")
    }
}
```

## 🔍 Code Quality

- ✅ **VibeCoding Compliant**: Follows AGENT.md guidelines
- ✅ **Separation of Concerns**: Service/System/Scene split
- ✅ **Plugin Pattern**: Custom plugins with derive macro
- ✅ **Hook System**: Extensible without core changes
- ✅ **Type Safety**: Strongly typed events and states

## 📚 Related Documentation

- [`AGENT.md`](../../AGENT.md) - VibeCoding philosophy
- [`PLUGIN_LIST.md`](../../docs/PLUGIN_LIST.md) - Built-in plugins
- [`docs/BEST_PRACTICES.md`](../../docs/BEST_PRACTICES.md) - Plugin development guide

---

**Built with ISSUN (一寸)** - VibeCoding for game logic 🎧
