# Pandemic Crisis

A turn-based pandemic management game demonstrating the Contagion Plugin's infection state machine and graph-based propagation.

## Overview

Race against time to contain a deadly disease spreading across a global city network. Balance quarantines, awareness campaigns, and cure research to save humanity.

## Features

- **Infection State Machine**: Incubating (20%) → Active (80%) → Recovered (5%) → Plain (0%)
- **Global City Network**: 8 major cities connected by travel routes
- **Strategic Actions**: Quarantine, awareness, cure research, emergency healthcare
- **Dynamic Mutations**: Disease evolves during transmission
- **Win/Lose Conditions**: Multiple victory and defeat scenarios

## How to Play

```bash
cargo run -p pandemic-crisis
```

### Available Actions

- **[1] Quarantine City** (3 AP): Block disease transmission for 3 turns
- **[2] Increase Awareness** (2 AP): Educate population, increase resistance
- **[3] Develop Cure Research** (5 AP): Progress toward cure (need 100%)
- **[4] Emergency Healthcare** (4 AP): Boost city resistance temporarily
- **[5] Travel Ban** (2 AP): Reduce global transmission rates
- **[6] Monitor City** (1 AP): Reveal infection details
- **[7] End Turn**: Advance to next turn

### Victory Conditions

1. **Cure Deployed**: Research cure (100%) + deploy to all cities
2. **Natural Containment**: Keep infections <10% for 15 turns

### Defeat Conditions

1. **Global Pandemic**: 70%+ of population infected
2. **Critical Mutations**: 5+ Critical severity variants
3. **Economic Collapse**: 50%+ cities quarantined for 10+ turns

## Plugin Usage

This example demonstrates:

- **ContagionPlugin**: State-based disease propagation
- **ActionPlugin**: Player action point management
- **TimePlugin**: Turn-based progression
- **MetricsPlugin**: Statistics tracking

## Code Structure

```
src/
├── main.rs              # Entry point + game loop
├── world.rs             # City network setup
├── disease.rs           # Disease configuration
├── player.rs            # Player actions
├── game_rules.rs        # Win/lose conditions
├── display.rs           # CLI rendering
└── events.rs            # Event handlers
```

## Design Document

See `workspace/pandemic_game_design.md` for full game design specification.
