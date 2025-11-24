# issun-cli

Command-line interface for ISSUN game framework.

## Installation

From the project root:

```bash
cargo install --path crates/issun-cli
```

Or run directly:

```bash
cargo run -p issun-cli -- [COMMAND]
```

## Usage

```bash
# Show help
issun --help

# Show help for analyze command
issun analyze --help

# List all plugins
issun analyze --list-plugins

# Show detailed plugin information
issun analyze --plugin-details

# Show details for specific plugins
issun analyze --plugin-details --plugins combat,inventory,loot

# Generate event flow graph
issun analyze --event-flow

# Generate hook flow graph
issun analyze --hook-flow --max-plugins 5

# Generate combined event + hook graph
issun analyze --combined-flow --max-plugins 3

# Validate event consistency
issun analyze --validate

# Combine multiple operations
issun analyze --list-plugins --validate --hook-flow
```

## Global Options

- `-C, --project-root <PATH>` - Path to project root directory (default: `.`)
- `--plugin-dir <PATH>` - Plugin directory relative to project root (default: `crates/issun/src/plugin`)
- `-o, --output <PATH>` - Output directory for generated files (default: `.`)

## Analyze Command

The `analyze` command provides static analysis of ISSUN plugin architecture.

### Options

- `--list-plugins` - List all plugins with summary information
- `--plugin-details` - Show detailed information about plugins
- `--plugins <NAMES>` - Filter by specific plugins (comma-separated)
- `--max-plugins <N>` - Maximum number of plugins to show in graphs (default: 5)

### Graph Generation

- `--event-flow` - Generate event flow graph (subscriptions/publications)
- `--hook-flow` - Generate hook flow graph (trait dependencies)
- `--combined-flow` - Generate combined event + hook graph
- `-o, --output <FILE>` - Output file path (default: `<type>_flow.mmd`)

Generated graphs are in Mermaid format (`.mmd`) and can be visualized at https://mermaid.live

### Validation

- `--validate` - Validate event consistency
  - Detects unused events (published but not subscribed)
  - Detects missing publishers (subscribed but not published)
  - Detects duplicate subscriptions
  - Detects potential event loops (circular dependencies)

## Examples

### Basic Analysis

```bash
# Analyze all plugins
issun analyze --list-plugins

# Output:
# 📦 Plugin List:
#   • combat - System: CombatSystem (subs: 3, pubs: 0, hooks: 1) | 1 hook traits
#   • inventory - System: InventorySystem (subs: 4, pubs: 0, hooks: 1) | 1 hook traits
#   • loot - System: LootSystem (subs: 2, pubs: 0, hooks: 1) | 1 hook traits
#   ...
```

### Detailed Plugin Information

```bash
# Show details for specific plugins
issun analyze --plugin-details --plugins combat,inventory

# Output:
# 📦 Plugin: combat
#    Path: ./crates/issun/src/plugin/combat
#
#    ⚙️ System: CombatSystem
#       Module: plugin::combat::system
#
#       📨 Subscribes to:
#          • CombatStartRequested
#          • CombatTurnAdvanceRequested
#          • CombatEndRequested
#
#       🪝 Uses Hooks:
#          • CombatHook
#
#    🪝 Hook Traits:
#       • CombatHook (5 methods)
```

### Generate Graphs

```bash
# Generate hook flow graph for first 3 plugins
issun analyze --hook-flow --max-plugins 3 -o hook_flow.mmd

# Generate combined graph for specific plugins
issun analyze --combined-flow --plugins combat,inventory,loot -o my_game_flow.mmd
```

### Validate Event Flows

```bash
issun analyze --validate

# Output:
# 🔎 Validating Event Flow...
#
# ✅ No validation warnings found!
#
# 📋 Validation Summary:
#    🔴 High severity: 0
#    🟡 Medium severity: 0
#    🟢 Low severity: 0
```

## Architecture

```
issun-cli/
├── src/
│   ├── main.rs           # CLI entry point with clap
│   ├── error.rs          # Error types
│   ├── config.rs         # Configuration
│   └── commands/
│       ├── mod.rs        # Command exports
│       └── analyze.rs    # Analyze command implementation
└── Cargo.toml
```

## Future Commands

The CLI is designed to be extensible. Potential future commands:

- `issun new <name>` - Create new game project from template
- `issun run` - Run game in development mode
- `issun test` - Run game tests with coverage
- `issun build` - Build optimized release binary
- `issun deploy` - Deploy to server or package for distribution

## Contributing

The CLI uses:
- `clap` for argument parsing with derive API
- `issun-analyzer` for static analysis
- `thiserror` for error handling
- Modular command structure for easy extension

To add a new command:
1. Create `src/commands/your_command.rs`
2. Implement command with `#[derive(Args)]`
3. Add to `Commands` enum in `main.rs`
4. Export in `commands/mod.rs`
