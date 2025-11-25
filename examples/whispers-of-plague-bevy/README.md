# Whispers of Plague - Bevy Edition

A **plague simulation game** rebuilt with **Bevy ECS** and **Ratatui** for terminal UI.

## 🎮 Game Concept

Play as either a **Plague** (spread infection) or **Savior** (save the city) across 20 turns. Watch as the virus spreads through a city graph topology with mutation mechanics.

### Game Modes

- **🦠 Plague Mode**: Spread infection across districts. Win by infecting >70% of the population.
- **💉 Savior Mode**: Save the city from disease. Win by keeping >60% healthy until turn 20.

## 🏗️ Architecture (Bevy ECS)

This example demonstrates **Bevy's ECS architecture**:

### Bevy Plugins

```
whispers-of-plague-bevy/
├── src/
│   ├── main.rs              # Entry point
│   ├── states.rs            # GameScene state
│   ├── components.rs        # District & Contagion components
│   ├── resources.rs         # Global resources
│   ├── systems.rs           # Game logic systems
│   ├── plugins.rs           # Bevy plugins
│   └── ui.rs                # Ratatui rendering
└── Cargo.toml
```

### State Machine

```rust
#[derive(States)]
pub enum GameScene {
    Title,   // Mode selection
    Game,    // Main gameplay
    Result,  // Victory/Defeat screen
}
```

### Components (ECS Entities)

- **District**: Population, infection count, panic level
- **Contagion**: Disease entity with severity level

### Resources (Global State)

- **GameContext**: Turn number, max turns, game mode
- **ContagionGraph**: Graph topology (edges between districts)
- **VictoryResult**: Win/loss state
- **UIState**: Message log

### Systems (Game Logic)

1. **Input Systems**: Handle keyboard input per scene
2. **Spread Contagion System**: Simulate disease spread through graph
3. **Mutate Virus System**: Upgrade virus severity at infection thresholds
4. **Win Condition System**: Check victory/defeat conditions
5. **UI Render System**: Draw terminal UI with Ratatui

## 🚀 Running the Game

```bash
cd examples/whispers-of-plague-bevy
cargo run
```

**Controls**:

**Title Screen**:
- **1**: Select Plague mode
- **2**: Select Savior mode
- **Q**: Quit

**Game Screen**:
- **1-5**: Select district (Downtown, Industrial, Residential, Suburbs, Harbor)
- **N**: Next turn
- **R**: Spread rumor in selected district (+20% panic)
- **I**: Isolation policy in selected district (-10% infections, -15% panic)
- **Q**: Quit

**Result Screen**:
- **ENTER**: Return to title
- **Q**: Quit

## 📊 Game Mechanics

### Player Actions

**Spread Rumor (R)**:
- Increases panic level by 20% in selected district
- Higher panic → faster infection spread
- Use in Plague mode to accelerate disease
- Max panic: 100%

**Isolation Policy (I)**:
- Reduces infections by 10% in selected district
- Decreases panic level by 15%
- Use in Savior mode to control outbreak
- Immediate effect on current turn

### Virus System

- **Mutations**: Virus mutates based on total infections
  - Alpha (0-5k): spread_rate=0.35, lethality=0.05
  - Beta (5k-10k): spread_rate=0.45 (+30%), lethality=0.06 (+20%)
  - Gamma (10k+): spread_rate=0.53 (+50%), lethality=0.07 (+40%)

### Spread Formula

```rust
spread_count = infected × edge_rate × virus_spread_rate × (1.0 + panic_level)
```

### District Model

```rust
pub struct District {
    pub population: u32,
    pub infected: u32,
    pub dead: u32,
    pub panic_level: f32,  // 0.0-1.0
}
```

### Graph Topology

5 districts connected by edges with different transmission rates:
- Downtown ↔ Industrial (0.3)
- Downtown ↔ Residential (0.4)
- Downtown ↔ Harbor (0.2)
- Residential ↔ Suburbs (0.5)
- etc.

## 🎓 Learning Points

### 1. **Bevy States**

```rust
app.init_state::<GameScene>()
   .add_systems(Update, system.run_if(in_state(GameScene::Game)))
```

### 2. **Bevy Resources**

```rust
#[derive(Resource)]
pub struct GameContext {
    pub turn: u32,
    pub mode: GameMode,
}
```

### 3. **Bevy Components**

```rust
#[derive(Component)]
pub struct District {
    pub population: u32,
    pub infected: u32,
}
```

### 4. **Bevy Systems**

```rust
fn spread_contagion_system(
    mut districts: Query<&mut District>,
    graph: Res<ContagionGraph>,
) {
    // Game logic here
}
```

### 5. **System Ordering**

```rust
app.add_systems(Update, (
    spread_contagion_system,
    mutate_virus_system.after(spread_contagion_system),
    check_win_system.after(mutate_virus_system),
))
```

### 6. **State Transitions**

```rust
fn check_victory(mut next_state: ResMut<NextState<GameScene>>) {
    next_state.set(GameScene::Result);
}
```

## 📝 Extending the Game

### Add New District

```rust
commands.spawn(District::new("airport", "Airport", 6000));
```

### Add New Edge

```rust
ContagionGraph {
    edges: vec![
        Edge { from: "airport", to: "downtown", rate: 0.5 },
    ]
}
```

### Add New System

```rust
fn quarantine_system(mut districts: Query<&mut District>) {
    // Reduce transmission rate
}

app.add_systems(Update, quarantine_system.run_if(in_state(GameScene::Game)));
```

## 🔍 Bevy vs ISSUN

| Aspect | ISSUN | Bevy |
|--------|-------|------|
| **ECS Maturity** | Custom | Industry Standard |
| **Performance** | Unoptimized | Highly Optimized |
| **Parallelism** | async/await | Automatic Parallelization |
| **Ecosystem** | Custom | Massive crates.io |
| **State Machine** | Scene enum | Built-in States |
| **System Ordering** | Manual | Declarative with `.after()` |

---

**Built with Bevy ECS** - Industrial-strength game architecture 🦀
