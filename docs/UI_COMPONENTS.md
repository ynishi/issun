# UI Component System Best Practices

Complete guide to building composable, maintainable UIs with ISSUN's Component system.

---

## 🎨 Architecture Overview

ISSUN's UI system follows a **trait-based, backend-independent** architecture with three layers:

### Layer 1: Abstract Traits (Backend-Independent)
- `Component` - Single-resource components
- `MultiResourceComponent` - Multi-resource components
- `UILayer` - Layout abstraction
- `Theme` - Styling abstraction

### Layer 2: Ratatui Implementation
- `HeaderComponent`, `DistrictsComponent`, `LogComponent`, `StatisticsComponent`
- `RatatuiLayer`, `RatatuiTheme`
- Concrete widget rendering

### Layer 3: Rendering Macros
- `drive!` - Sequential component rendering with automatic layout
- `drive_to!` - Explicit area assignment with fallback support

---

## 📦 Component Trait

### When to Use Component vs MultiResourceComponent

**Use `Component`** (Recommended):
- ✅ Depends on a single primary resource type
- ✅ Automatic type inference
- ✅ Simpler API
- ✅ Example: `HeaderComponent<GameContext>`

**Use `MultiResourceComponent`**:
- ✅ Depends on multiple resources
- ✅ More flexible resource access
- ✅ Example: Components that need both `GameContext` and `CityMap`

---

## 🛠️ Implementing Component Traits

### Example: HeaderComponent

**Step 1: Implement the context trait**

```rust
use issun::ui::ratatui::HeaderContext;

impl HeaderContext for MyGameContext {
    fn turn(&self) -> u32 {
        self.current_turn
    }

    fn max_turns(&self) -> u32 {
        self.total_turns
    }

    fn mode(&self) -> String {
        format!("{:?}", self.game_mode)
    }
}
```

**Step 2: Use the component**

```rust
use issun::ui::ratatui::HeaderComponent;
use issun::ui::core::Component;

let header = HeaderComponent::<MyGameContext>::new();
if let Some(widget) = header.render(resources) {
    frame.render_widget(widget, area);
}
```

---

### Example: DistrictsComponent

**Step 1: Implement DistrictData**

```rust
use issun::ui::ratatui::DistrictData;

impl DistrictData for District {
    fn id(&self) -> &str {
        &self.district_id
    }

    fn name(&self) -> &str {
        &self.district_name
    }

    fn format_line(&self) -> String {
        format!("🏙️ {}: {} citizens", self.name(), self.population)
    }
}
```

**Step 2: Implement DistrictsProvider**

```rust
use issun::ui::ratatui::DistrictsProvider;

impl DistrictsProvider for CityMap {
    type District = District;

    fn districts(&self) -> &[Self::District] {
        &self.all_districts
    }
}
```

**Step 3: Use the component**

```rust
use issun::ui::ratatui::DistrictsComponent;

let districts = DistrictsComponent::<CityMap>::new();
if let Some(widget) = districts.render_with_selection(resources, selected_idx) {
    frame.render_widget(widget, area);
}
```

---

## 🚀 Using the drive! Macro

The `drive!` macro simplifies rendering by handling layout and error checking automatically.

### Basic Usage

```rust
use issun::{drive, drive_to};
use issun::ui::ratatui::*;
use issun::ui::layer::UILayoutPresets;

fn render_game(frame: &mut Frame, resources: &ResourceContext, selected: usize) {
    let header = HeaderComponent::<GameContext>::new();
    let districts = DistrictsComponent::<CityMap>::new();
    let log = LogComponent::<GameLog>::new();

    drive! {
        frame: frame,
        layout: RatatuiLayer::three_panel().apply(frame.area()),
        [
            header.render(resources),
            districts.render_with_selection(resources, selected),
            log.render_multi(resources),
        ]
    }
}
```

**Benefits:**
- ✅ Automatic Option<Widget> handling
- ✅ Sequential rendering to layout chunks
- ✅ Clean, declarative syntax
- ✅ ~60% less boilerplate code

---

### Using drive_to! with Fallback Widgets

```rust
use issun::drive_to;

fn render_with_fallback(frame: &mut Frame, resources: &ResourceContext) {
    let layout = RatatuiLayer::two_column(50).apply(frame.area());
    let header = HeaderComponent::<GameContext>::new();

    // Fallback widget when resource is missing
    let fallback = Paragraph::new("Loading...")
        .block(Block::default().borders(Borders::ALL).title("Status"));

    drive_to! {
        frame: frame,
        [
            (layout[0], header.render(resources), fallback),
        ]
    }
}
```

---

## 📐 Layout Management

### UILayer Presets

```rust
use issun::ui::layer::UILayoutPresets;
use issun::ui::ratatui::RatatuiLayer;

// Three-panel layout (header, main, footer)
let layout = RatatuiLayer::three_panel().apply(area);

// Two-column layout (60% left, 40% right)
let layout = RatatuiLayer::two_column(60).apply(area);

// Sidebar layout (20% left, 80% right)
let layout = RatatuiLayer::sidebar().apply(area);

// Detail layout (80% left, 20% right)
let layout = RatatuiLayer::detail().apply(area);
```

### Custom Layouts

```rust
use issun::ui::layer::{LayoutConstraint, LayoutDirection, UILayer};
use issun::ui::ratatui::RatatuiLayer;

let custom = RatatuiLayer::new(
    "custom_layout",
    LayoutDirection::Vertical,
    vec![
        LayoutConstraint::Length(5),      // Fixed height
        LayoutConstraint::Percentage(60), // 60% of remaining
        LayoutConstraint::Min(10),        // At least 10 lines
    ],
).apply(area);
```

---

## 🎨 Theme System

### Using Predefined Themes

```rust
use issun::ui::ratatui::RatatuiTheme;
use issun::ui::theme::ThemePresets;

// Plague theme (red-based, ominous)
let theme = RatatuiTheme::plague();

// Savior theme (blue-based, hopeful)
let theme = RatatuiTheme::savior();

// Standard presets
let dark = RatatuiTheme::dark();
let light = RatatuiTheme::light();
let high_contrast = RatatuiTheme::high_contrast();

// Use theme for styling
let style = theme.style_primary();
let selected_style = theme.style_selected();
```

---

## ✅ Best Practices

### 1. Separate Component Traits from Game Logic

**GOOD:**
```rust
// ui/components.rs - Component trait implementations only
impl HeaderContext for GameContext { ... }
impl DistrictData for District { ... }

// models/context.rs - Game logic
pub struct GameContext {
    pub turn: u32,
    pub max_turns: u32,
    pub mode: GameMode,
}
```

**BAD:**
```rust
// Mixing UI traits in game logic file
pub struct GameContext {
    pub turn: u32,
    // ... game logic mixed with UI concerns
}

impl HeaderContext for GameContext { ... } // ❌ Don't mix!
```

---

### 2. Use Helper Functions for Complex Rendering

When a component needs data that isn't in ResourceContext:

**GOOD:**
```rust
// ui/components.rs
pub fn statistics_lines(context: &GameContext, city: &CityMap) -> Vec<Line<'static>> {
    vec![
        Line::from(format!("Turn: {}/{}", context.turn, context.max_turns)),
        Line::from(format!("Population: {}", city.total_population())),
    ]
}

// ui/mod.rs
let stats_text = if let (Some(ctx), Some(city)) = (ctx, city) {
    statistics_lines(ctx, city)
} else {
    vec![Line::from("Loading...")]
};
```

**BAD:**
```rust
// Trying to force non-resource data into Component
struct GameStatistics<'a> { // ❌ Lifetime issues!
    context: &'a GameContext,
    city: &'a CityMap,
}
```

---

### 3. Always Provide Fallback Widgets

Resources may be unavailable during scene transitions:

**GOOD:**
```rust
if let Some(widget) = component.render(resources) {
    frame.render_widget(widget, area);
} else {
    let fallback = Paragraph::new("Loading...")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(fallback, area);
}
```

**BETTER:** Use `drive_to!` for automatic fallback handling:

```rust
drive_to! {
    frame: frame,
    [
        (area, component.render(resources), fallback_widget),
    ]
}
```

---

### 4. Keep format_line() Simple and Focused

**GOOD:**
```rust
impl DistrictData for District {
    fn format_line(&self) -> String {
        let emoji = self.get_emoji();
        let panic_bar = self.render_panic_bar();
        format!(
            "{} {}: {} infected | Panic: {} {}%",
            emoji, self.name, self.infected, panic_bar, self.panic_pct()
        )
    }
}
```

**BAD:**
```rust
impl DistrictData for District {
    fn format_line(&self) -> String {
        // ❌ Don't add complex business logic here
        self.calculate_infection_rate();
        self.update_panic_level();
        // ... 50 lines of logic
    }
}
```

---

## 🚫 Common Anti-Patterns

### Anti-Pattern 1: Over-using Components

**Problem:**
```rust
// ❌ Creating component for every tiny widget
struct TinyLabelComponent;
struct SingleNumberComponent;
```

**Solution:**
```rust
// ✅ Use components for logical UI sections only
// Render simple widgets directly
let label = Paragraph::new("Simple text");
frame.render_widget(label, area);
```

**Rule of Thumb:** Use Components when:
- Widget depends on ResourceContext
- Widget has reusable logic (multiple games/scenes)
- Widget has complex rendering (>20 lines)

---

### Anti-Pattern 2: Storing Scene Data in ResourceContext

**Problem:**
```rust
// ❌ GameSceneData is scene-specific, not global
resources.insert(GameSceneData { selected: 0, logs: vec![] });
```

**Solution:**
```rust
// ✅ Pass scene data directly
fn render_game(frame: &mut Frame, resources: &ResourceContext, data: &GameSceneData) {
    // Use data directly, not from resources
    let log_items: Vec<ListItem> = data.log_messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();
}
```

**Rule:** Only put **global game state** in ResourceContext, not **scene-specific UI state**.

---

### Anti-Pattern 3: Ignoring Option Returns

**Problem:**
```rust
// ❌ Unwrapping can panic
let widget = component.render(resources).unwrap();
frame.render_widget(widget, area);
```

**Solution:**
```rust
// ✅ Always handle None gracefully
if let Some(widget) = component.render(resources) {
    frame.render_widget(widget, area);
} else {
    // Show fallback or skip
}

// ✅ BETTER: Use drive_to! macro
drive_to! {
    frame: frame,
    [(area, component.render(resources), fallback)],
}
```

---

## 📊 Real-World Example: whispers-of-plague Refactoring

### Before: Monolithic Rendering (173 lines)

```rust
fn render_game(
    frame: &mut Frame,
    ctx: Option<&PlagueGameContext>,
    city: Option<&CityMap>,
    contagion_state: Option<&ContagionState>,
    data: &GameSceneData,
) {
    // Manual layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([...])
        .split(area);

    // Manual header rendering
    let header_text = if let Some(ctx) = ctx {
        format!("Turn {}/{} | Mode: {:?}", ctx.turn, ctx.max_turns, ctx.mode)
    } else {
        "Loading...".into()
    };
    // ... 50 more lines of manual rendering

    // Manual districts rendering
    let districts_items: Vec<ListItem> = if let Some(city) = city {
        city.districts.iter().enumerate().map(|(i, d)| {
            let emoji = match d.id.as_str() { ... };
            let panic_bar = "█".repeat(...) + &"░".repeat(...);
            // ... 40 more lines
        }).collect()
    } else {
        vec![ListItem::new("No data")]
    };
    // ... 80 more lines
}
```

### After: Component-Based Rendering (106 lines)

```rust
fn render_game(frame: &mut Frame, resources: &ResourceContext, data: &GameSceneData) {
    // Layout management
    let main_layout = RatatuiLayer::three_panel().apply(frame.area());

    // Components
    let header = HeaderComponent::<PlagueGameContext>::new();
    let districts = DistrictsComponent::<CityMap>::new();

    // Render with automatic error handling
    drive_to! {
        frame: frame,
        [
            (main_layout[0], header.render(resources), header_fallback),
        ]
    }

    if let Some(widget) = districts.render_with_selection(resources, data.selected_district) {
        frame.render_widget(widget, main_chunks[0]);
    }

    // ... clean, composable rendering
}
```

**Results:**
- ✅ **38.7% code reduction** (67 lines saved)
- ✅ **Reusable components** across scenes
- ✅ **Easier to test** (components in isolation)
- ✅ **Better error handling** (automatic fallback)
- ✅ **Type-safe** (compile-time checking)

---

## 📚 See Also

- `crates/issun/src/ui/` - UI component source code
- `crates/issun/examples/drive_macro_demo.rs` - Drive macro examples
- `examples/whispers-of-plague/src/ui/` - Real-world usage
- `docs/BEST_PRACTICES.md` - General best practices
