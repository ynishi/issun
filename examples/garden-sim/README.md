# Garden Simulator

A simple garden management game demonstrating **GenerationPlugin** (growth) and **EntropyPlugin** (decay) working together in harmony.

## Concept

🌱 **Plants grow** using `GenerationPlugin` (0% → 100%)
🍂 **Plants decay** using `EntropyPlugin` (100% → 0%)

Players must manage the balance between growth and decay to successfully cultivate their garden.

## Features

- **5 Plant Species**: Tomato 🍅, Lettuce 🥬, Carrot 🥕, Wheat 🌾, Sunflower 🌻
- **Growth Stages**: Seed → Seedling → Growing → Mature → Ready
- **Health System**: Healthy → Good → Stressed → Dying → Dead
- **Parallel Processing**: Uses Rayon for high-performance ECS updates
- **Real-time Simulation**: Visual feedback every 5 ticks

## Plant Properties

| Species | Growth Rate | Decay Rate | Max Health | Harvest Yield |
|---------|-------------|------------|------------|---------------|
| 🍅 Tomato | 2.0 | 0.3 | 100 | 3 |
| 🥬 Lettuce | 5.0 | 0.5 | 80 | 2 |
| 🥕 Carrot | 3.0 | 0.2 | 120 | 4 |
| 🌾 Wheat | 4.0 | 0.3 | 100 | 5 |
| 🌻 Sunflower | 1.5 | 0.1 | 150 | 1 |

**Strategy**:
- **Lettuce**: Fast growth but fragile (high decay)
- **Sunflower**: Slow growth but hardy (low decay)
- **Carrot**: Balanced and hardy (good for beginners)

## Running the Example

```bash
cd examples/garden-sim
cargo run
```

## How It Works

### GenerationPlugin (Growth)
```rust
Generation::new(
    species.max_growth(),      // Target: 100.0
    species.growth_rate(),     // Progress per tick
    GenerationType::Organic,   // Affected by temperature, fertility, light
)
```

**Environmental Factors**:
- Temperature: 22°C (optimal)
- Fertility: 0.8 (rich soil)
- Resources: 1.0 (well-watered)
- Light: 0.9 (full sun)

### EntropyPlugin (Decay)
```rust
Durability::new(
    species.max_durability(),  // Max health
    species.decay_rate(),      // Decay per tick
    MaterialType::Organic,     // Organic materials decay faster
)
```

**Environmental Exposure**:
- Humidity: 0.5 (moderate)
- Pollution: 0.0 (clean air)
- Temperature: 22°C
- Sunlight: 0.9

### Update Loop
```rust
// Each tick (200ms)
1. GenerationSystem: Plants grow based on environment
2. EntropySystem: Plants decay based on exposure
3. Cleanup: Remove dead/completed plants
4. Display: Show garden status
```

## Example Output

```
🌻 Welcome to Garden Simulator!
Demonstrating GenerationPlugin + EntropyPlugin

🌱 Planted 🍅 Tomato (growth: 2.0/tick, decay: 0.3/tick)
🌱 Planted 🥬 Lettuce (growth: 5.0/tick, decay: 0.5/tick)
🌱 Planted 🥕 Carrot (growth: 3.0/tick, decay: 0.2/tick)
🌱 Planted 🌾 Wheat (growth: 4.0/tick, decay: 0.3/tick)
🌱 Planted 🌻 Sunflower (growth: 1.5/tick, decay: 0.1/tick)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌻 GARDEN STATUS - Tick #5
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. 🍅 Tomato - Growth: ◐ 45.2% | Health: Healthy 95.3%
2. 🥬 Lettuce - Growth: ◑ 78.9% | Health: Good 92.1%
3. 🥕 Carrot - Growth: ◑ 62.3% | Health: Healthy 96.8%
4. 🌾 Wheat - Growth: ◑ 71.5% | Health: Healthy 95.0%
5. 🌻 Sunflower - Growth: ◐ 32.8% | Health: Healthy 98.5%

📊 Metrics:
  Generation: 5 entities, 12.45 total progress
  Entropy: 5 entities, 1.23 total decay
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Architecture

```
Garden Simulator
├── GenerationPlugin
│   ├── GenerationState (hecs::World)
│   ├── GenerationSystem (parallel updates)
│   ├── GenerationConfig (environment modifiers)
│   └── GardenGenerationHook (custom logic)
├── EntropyPlugin
│   ├── EntropyState (hecs::World)
│   ├── EntropySystem (parallel updates)
│   ├── EntropyConfig (decay modifiers)
│   └── GardenEntropyHook (custom logic)
└── Game Logic
    ├── PlantSpecies (5 types)
    ├── GrowthStage (5 stages)
    ├── PlantHealth (5 levels)
    └── Garden (main controller)
```

## Future Enhancements

- [ ] Interactive TUI (watering, fertilizing, harvesting)
- [ ] Weather system (rain, drought, seasons)
- [ ] Pests and diseases
- [ ] Crop rotation and soil depletion
- [ ] Market system (sell harvested crops)
- [ ] Achievement system
- [ ] Save/load garden state

## Learning Points

This example demonstrates:
1. **Dual Plugin Integration**: Combining GenerationPlugin + EntropyPlugin
2. **ECS Performance**: Parallel processing of multiple entities
3. **Custom Hooks**: Game-specific behavior customization
4. **Environmental Factors**: How modifiers affect generation/decay rates
5. **Component-Based Design**: Separation of growth and health systems

---

**Enjoy gardening! 🌻**
