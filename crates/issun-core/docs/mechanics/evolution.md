# Natural Evolution Mechanic - Design Document

## Overview

The **Natural Evolution Mechanic** is a unified system that models time-based natural state changes in entities. It merges two previously separate concepts—**Entropy** (decay, degradation, reduction) and **Generation** (growth, accumulation, production)—into a single, flexible mechanic with three policy dimensions.

### Target Phenomena

- Food rotting (decay)
- Plants growing (growth)
- Resources regenerating (growth)
- Equipment degrading (decay)
- Population dynamics (cyclic)
- Seasonal changes (oscillating)

## Architecture

### Policy-Based Design

Three orthogonal policy dimensions that can be freely combined:

```rust
EvolutionMechanic<DirectionPolicy, EnvironmentalPolicy, RateCalculationPolicy>
```

### Three Policy Dimensions

#### 1. DirectionPolicy - Change Direction

Determines the **direction** of state change.

**Standard Implementations:**
- `Growth` - Positive direction (value increases over time)
- `Decay` - Negative direction (value decreases over time)
- `Cyclic` - Bidirectional based on thresholds (switches between growth/decay)
- `Oscillating` - Sinusoidal pattern (seasonal, circadian rhythms)

**Responsibility:** Calculate directional multiplier from current value, bounds, and elapsed time.

#### 2. EnvironmentalPolicy - Environmental Influence

Determines how **environmental factors** influence the evolution rate.

**Standard Implementations:**
- `NoEnvironment` - No environmental influence (multiplier = 1.0)
- `TemperatureBased` - Affected by temperature
- `HumidityBased` - Affected by humidity
- `ComprehensiveEnvironment` - Considers multiple factors

**Responsibility:** Calculate environmental multiplier [0.0, ∞)
- 0.0 = evolution completely halted
- 1.0 = normal evolution rate
- >1.0 = accelerated evolution

#### 3. RateCalculationPolicy - Rate Scaling

Determines **how the rate scales** with current value.

**Standard Implementations:**
- `LinearRate` - Constant rate of change
- `ExponentialRate` - Rate proportional to current value (compound growth/decay)
- `ThresholdRate` - Rate changes at specific thresholds
- `DiminishingRate` - Rate decreases as value approaches limits

**Responsibility:** Calculate actual change amount from base rate, current value, direction multiplier, and environmental multiplier.

## Core Types

### EvolutionConfig
Global configuration shared across all entities:
- `base_rate` - Base evolution rate per tick
- `time_delta` - Time delta per step

### EvolutionState
Per-entity state:
- `value` - Current value (e.g., food freshness, plant size)
- `min` / `max` - Boundary values
- `rate_multiplier` - Entity-specific rate multiplier
- `subject` - Type of evolving subject
- `status` - Current status (Active/Paused/Completed/Depleted)

### EvolutionInput
Input for a single evolution step:
- `time_delta` - Time elapsed since last update
- `environment` - Environmental conditions affecting this entity

### EvolutionEvent
Events emitted during evolution:
- `ValueChanged` - Value changed
- `MinimumReached` / `MaximumReached` - Boundary reached
- `ThresholdCrossed` - Threshold crossed
- `StatusChanged` - Status changed

## Preset Configurations

Each preset is an optimized policy combination for specific use cases:

| Preset | Direction | Environment | Rate | Use Case |
|--------|-----------|-------------|------|----------|
| **OrganicGrowth** | Growth | TemperatureBased | Linear | Plants, bacteria, organic matter |
| **FoodDecay** | Decay | HumidityBased | Exponential | Perishable items, organic materials |
| **ResourceRegeneration** | Growth | NoEnvironment | Diminishing | Mana, stamina, renewable resources |
| **EquipmentDegradation** | Decay | NoEnvironment | Linear | Weapons, armor, tools |
| **PopulationDynamics** | Cyclic | Comprehensive | Threshold | Animal populations, NPC demographics |
| **SeasonalCycle** | Oscillating | NoEnvironment | Linear | Day/night cycles, seasonal effects |

## Usage Pattern

### Basic Flow

```rust
// 1. Define mechanic type (policy combination)
type FoodSpoilage = EvolutionMechanic<Decay, HumidityBased, ExponentialRate>;

// 2. Global configuration
let config = EvolutionConfig { base_rate: 1.0, time_delta: 1.0 };

// 3. Entity state
let mut state = EvolutionState { value: 100.0, min: 0.0, max: 100.0, ... };

// 4. Input (with environmental conditions)
let input = EvolutionInput { time_delta: 1.0, environment: ... };

// 5. Execute step
FoodSpoilage::step(&config, &mut state, input, &mut event_emitter);

// 6. Handle events
// event_emitter receives ValueChanged, MinimumReached, etc.
```

### Bevy Integration

```rust
app.add_plugins(EvolutionPluginV2::<FoodSpoilage>::default());
// Components: EvolutionState, EvolutionInput
// Messages: EvolutionTick (trigger), EvolutionEvent (results)
```

## Design Rationale

### Why Three Dimensions?

1. **DirectionPolicy Separation**
   - "Which way" and "how fast" are orthogonal concepts
   - Enables complex patterns (cyclic, oscillating)

2. **EnvironmentalPolicy Separation**
   - Environment-dependent systems (food, plants) and independent systems (equipment, abstract resources) share the same framework
   - Easy to add custom environmental factors

3. **RateCalculationPolicy Separation**
   - Mathematical progression model can be chosen independently
   - Same direction can have different growth curves (compound interest, diminishing returns)

### Benefits of Unification

1. **Single System**: No need to implement Entropy and Generation separately
2. **Code Reuse**: Common infrastructure for all time-based changes
3. **Composability**: Mix and match policies to create new behaviors
4. **Type Safety**: Compile-time guarantees about behavior
5. **Zero-Cost Abstraction**: No runtime overhead

### Backward Compatibility

Existing systems can be replaced with type aliases:

```rust
type EntropyMechanic = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;
type GenerationMechanic = EvolutionMechanic<Growth, NoEnvironment, LinearRate>;
```

## Related Mechanics

- **StateMachine**: Handles discrete state transitions (Incubating → Active)
- **Evolution**: Handles continuous value changes (freshness: 100 → 0)
- Cooperation: StateMachine triggers status changes, Evolution updates values

## Future Extensions

1. **Custom Direction Policies**: User-defined directional behaviors
2. **Multi-Factor Environment**: More sophisticated environmental models
3. **Event-Driven Rate Changes**: External events affecting rates
4. **Composite Policies**: Combine multiple policies with weights
5. **Predictive Queries**: "When will this food spoil?" style queries
