# Economy Plugin Refactoring Proposal (v2)

This document outlines the design for refactoring the economy-related parts of the `border-economy` example into a generic, reusable `issun` plugin. This version incorporates feedback to make the system event-driven and more configurable.

## 1. Decompose `GameContext` State into Granular Resources

The core principle remains the same: break down the God object into single-responsibility resources.

```rust
// In: crates/issun/src/plugin/economy/resources.rs

// Replaces GameContext.ledger
pub struct BudgetLedger { /* ... */ }

// Replaces GameContext.policies
pub struct PolicyDeck { /* ... */ }

// A resource to hold configuration for the economy system
#[derive(Clone)] // Clone is needed for the plugin to own it.
pub struct EconomyConfig {
    pub settlement_period_days: u32,
    pub dividend_base: i64,
    pub dividend_rate: f32,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            settlement_period_days: 7,
            dividend_base: 200,
            dividend_rate: 0.04,
        }
    }
}
```

## 2. Introduce a Time-keeping System and Events

To decouple the `EconomySystem` from time-keeping logic, we introduce a dedicated `GameClockSystem` and a `DayPassedEvent`. This makes the architecture event-driven instead of relying on polling.

```rust
// In a new time plugin or a core module:
// crates/issun/src/plugin/time/mod.rs

// The event published when a day ends.
pub struct DayPassedEvent;

// A resource to hold the game's clock.
pub struct GameClock {
    pub day: u32,
    pub actions_remaining: u32,
}

// A system responsible for advancing time and publishing the event.
#[derive(Default, DeriveSystem)]
#[system(name = "game_clock_system")]
pub struct GameClockSystem;

impl GameClockSystem {
    pub async fn advance_day(&mut self, resources: &mut ResourceContext) {
        let mut clock = resources.get_mut::<GameClock>().await.unwrap();
        clock.day += 1;
        // ... reset action points, etc.

        let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
        event_bus.publish(DayPassedEvent);
    }
}
```

## 3. Create a Stateless `EconomyService`

This remains unchanged. It's responsible for pure calculations.

```rust
// In: crates/issun/src/plugin/economy/service.rs

#[async_trait::async_trait]
pub trait EconomyService: Service {
    fn forecast_income(&self, territories: &TerritoryResource, ledger: &BudgetLedger) -> Currency;
    fn forecast_upkeep(&self, factions: &FactionResource, prototypes: &PrototypeResource, ledger: &BudgetLedger) -> Currency;
}

pub struct BuiltInEconomyService;
// ... implementation ...
```

## 4. Create an Event-Driven `EconomySystem`

The `EconomySystem` is now much cleaner. It no longer checks the day count. Instead, it subscribes to `DayPassedEvent` and runs its logic only when triggered.

```rust
// In: crates/issun/src/plugin/economy/system.rs

#[derive(Default, DeriveSystem)]
#[system(name = "economy_system")]
pub struct EconomySystem;

impl EconomySystem {
    // This method is called every frame by the game loop.
    pub async fn update(&mut self, services: &ServiceContext, resources: &mut ResourceContext) {
        let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
        if event_bus.reader::<DayPassedEvent>().is_empty() {
            return; // No event, do nothing.
        }

        // Event received, run settlement logic.
        self.run_settlement(services, resources).await;
    }

    async fn run_settlement(&mut self, services: &ServiceContext, resources: &mut ResourceContext) {
        let economy_service = services.get_as::<BuiltInEconomyService>("economy_service").unwrap();
        let clock = resources.get::<GameClock>().await.unwrap();
        let config = resources.get::<EconomyConfig>().await.unwrap();

        // The check is now based on the event, but we might still need the day for the modulo.
        if clock.day % config.settlement_period_days != 0 {
            return;
        }

        // ... rest of the settlement logic remains the same ...
        // It acquires locks on BudgetLedger, etc., and applies changes.
    }
}
```

## 5. Define the New Configurable `BuiltInEconomyPlugin`

The plugin is updated to accept a configuration struct in its constructor, improving ergonomics and consistency with other engine plugins.

```rust
// In: crates/issun/src/plugin/economy/mod.rs

pub struct BuiltInEconomyPlugin {
    config: EconomyConfig,
}

impl BuiltInEconomyPlugin {
    pub fn new(config: EconomyConfig) -> Self {
        Self { config }
    }
}

impl Default for BuiltInEconomyPlugin {
    fn default() -> Self {
        Self {
            config: EconomyConfig::default(),
        }
    }
}

impl Plugin for BuiltInEconomyPlugin {
    fn name(&self) -> &'static str {
        "issun:economy"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(BuiltInEconomyService::default()));
        builder.register_system(Box::new(EconomySystem::default()));

        // Register the config provided by the user.
        builder.register_resource(self.config.clone());

        // Register default state.
        builder.register_runtime_state(BudgetLedger::new(Currency::new(2400)));
        builder.register_runtime_state(PolicyDeck { /* ... */ });
        // GameClock would be registered by a separate TimePlugin.
    }
}

// Example of user code:
// let game = GameBuilder::new()
//     .with_plugin(BuiltInTimePlugin::default()) // Provides GameClock and DayPassedEvent
//     .with_plugin(BuiltInEconomyPlugin::new(EconomyConfig {
//         settlement_period_days: 30, // User-defined setting
//         ..Default::default()
//     }))
//     .build();
```

## Summary of Benefits (Updated)

*   **Decoupling**: The `EconomySystem` is now decoupled from the time-keeping mechanism, reacting to events rather than polling state.
*   **Configurability**: The plugin is easier to configure, following a consistent pattern used elsewhere in the engine.
*   **Clarity & Testability**: All previous benefits are retained and enhanced.

## Future Considerations

*   **`Cronic` System**: The idea of a generic, time-based event scheduling system (`Cronic`) is excellent. It could publish events like `WeeklyEvent`, `MonthlyEvent`, etc., further abstracting time-based logic for all plugins.