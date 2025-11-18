# Event Bus Design (Approach 2: Type-Safe)

## 1. Overview

This document outlines the design for a type-safe, `TypeId`-based event bus for the `issun` engine. The goal is to enable decoupled communication between different parts of the engine (e.g., `Systems`, `Scenes`) without requiring them to have direct knowledge of each other.

This follows the "publish-subscribe" pattern.

- **Publisher**: Any component can send an event of a specific type.
- **Subscriber**: Any component can read all events of a specific type that were sent within the current frame.

## 2. Core Concepts & API

The API will be inspired by Bevy's `Events` system, which is known for its simplicity and efficiency.

### Events

An event can be any struct that is `'static`. There is no need for a central enum.

```rust
// An event is just a simple struct
pub struct PlayerDamaged {
    pub target: EntityId, // Assuming we have an EntityId type
    pub amount: u32,
}

pub struct EnemyDefeated {
    pub source: EntityId,
    pub score_value: u32,
}
```

### EventBus (The Central Resource)

The `EventBus` is a runtime resource created automatically by `GameBuilder` and stored inside `ResourceContext`. Each event type gets its own channel that double-buffers events.

```rust
// crates/issun/src/event.rs
pub struct EventBus {
    channels: HashMap<TypeId, Box<dyn EventChannelStorage>>,
}

impl EventBus {
    /// Creates a new, empty EventBus.
    pub fn new() -> Self { ... }

    /// Publishes an event so it becomes visible after the next dispatch.
    pub fn publish<E: 'static + Send + Sync>(&mut self, event: E) { ... }

    /// Returns a reader over events of type E from the previous frame.
    pub fn reader<E: 'static + Send + Sync>(&mut self) -> EventReader<'_, E> { ... }

    /// Swaps channel buffers; called by the runner once per frame.
    pub fn dispatch(&mut self) { ... }
}
```

### EventReader

An `EventReader` is a lightweight struct that allows subscribers to iterate over the events of a specific type for the current frame.

```rust
pub struct EventReader<'a, E> {
    events: &'a [E],
    cursor: usize,
}

impl<'a, E> EventReader<'a, E> {
    pub fn iter(&self) -> impl Iterator<Item = &E> { ... }
    pub fn len(&self) -> usize { ... }
    pub fn is_empty(&self) -> bool { ... }
}
```

## 3. Data Structures (Internal)

To handle event storage and reading without complex borrowing issues, we will use a double-buffer approach for each event type.

### EventChannel<E>

This struct will manage the two buffers for a single event type `E`.

```rust
// Internal structure, not directly exposed to the user.
struct EventChannel<E: 'static> {
    // Events from the current frame are written here.
    a: Vec<E>,
    // Events from the previous frame are read from here.
    b: Vec<E>,
}

impl<E> EventChannel<E> {
    fn new() -> Self {
        Self {
            a: Vec::new(),
            b: Vec::new(),
        }
    }

    // Swaps the buffers and clears the new "a" buffer for the next frame.
    fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
        self.a.clear();
    }
}
```

The `EventBus` will hold a `HashMap<TypeId, Box<dyn Any + Send + Sync>>`, where the `Box<dyn Any>` contains an `EventChannel<E>`.

## 4. Integration Plan

### Step 1: `GameBuilder`

`GameBuilder` now inserts an `EventBus` into the `ResourceContext` automatically so every game and plugin can rely on it being present. No manual wiring is needed.

```rust
let mut resource_context = ResourceContext::new();
resource_context.insert(crate::event::EventBus::new());
```

### Step 2: `GameRunner`

`GameRunner::run` dispatches events once per loop iteration (after `Scene::on_update` and before the next frame starts). Because `ResourceContext` uses async locks, the runner awaits the write guard before calling `dispatch`.

```rust
if let Some(mut event_bus) = self.director.resources_mut().get_mut::<EventBus>().await {
    event_bus.dispatch();
}
```

### Step 3: Usage Example

#### Publishing an Event (e.g., in a `System`)

A system can acquire a mutable guard to `EventBus` and enqueue events for the next frame.

```rust
// In a combat system
async fn execute_attack(resources: &ResourceContext, damage: u32) {
    // ... logic ...

    if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
        event_bus.publish(PlayerDamaged { amount: damage });
    }
}
```

#### Reading Events (e.g., in a `Scene`'s update)

A scene can get access to the `ResourceContext` during its `on_update` lifecycle hook and read events.

```rust
// In a UI scene's on_update method
async fn on_update(&mut self, ..., resources: &mut ResourceContext) -> SceneTransition<Self> {
    if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
        let reader = event_bus.reader::<PlayerDamaged>();
        for event in reader.iter() {
            println!("Player took {} damage! Updating UI.", event.amount);
        }
    }
    SceneTransition::Stay
}
```

See `crates/issun/tests/event_bus_integration.rs` for an end-to-end example covering publish → dispatch → read.

## 5. File Structure

1.  **New File**: `crates/issun/src/event.rs` will contain the `EventBus`, `EventReader`, and `EventChannel` implementations.
2.  **Modification**: `crates/issun/src/lib.rs` will need to expose the new module (`pub mod event;`).
3.  **Modification**: `crates/issun/src/builder.rs` to initialize the `EventBus`.
4.  **Modification**: `crates/issun/src/engine/runner.rs` to call `dispatch()` in the game loop.
