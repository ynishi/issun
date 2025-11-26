# Lint Candidates for Best Practices

**Created**: 2025-11-26
**Purpose**: Identify patterns that should be enforced via static linting for issun-bevy

---

## ✅ Currently Enforced

### 1. Reflect Linting (`tests/lints.rs`)

**What**: All Bevy types must have proper Reflect derives
**Enforcement**: Static analysis via syn AST parsing
**Rules**:
- Components: `#[derive(Reflect)]` + `#[reflect(Component)]`
- Resources: `#[derive(Reflect)]` + `#[reflect(Resource)]`
- Messages: `#[derive(Reflect)]` + `#[reflect(opaque)]` (NOT `#[reflect(Message)]`)
- Events: `#[derive(Reflect)]` only (NOT `#[reflect(Event)]`)

**Status**: ✅ Implemented and working

---

## 🎯 Candidates to Add

### 2. Entity Query Safety (High Priority)

**Problem**: Using `.unwrap()` on entity queries causes panics when entities are despawned
**Pattern to Enforce**:
```rust
// ❌ FORBID in production code
fn bad_system(query: Query<&Health>) {
    let health = query.get(entity).unwrap();  // PANIC if deleted!
}

// ✅ REQUIRE in production code
fn good_system(query: Query<&Health>) {
    if let Ok(health) = query.get(entity) {
        // Safe
    }
}
```

**Scope**: All `src/**/*.rs` (excluding `#[cfg(test)]` blocks)

**Detection**:
- Pattern: `query.get(.*).unwrap()`
- Pattern: `query.get_mut(.*).unwrap()`
- Pattern: `query.get(.*).expect(.*)`

**Exemptions**: Test code (already verified safe)

**Estimated Effort**: Medium (AST parsing for unwrap after Query methods)

---

### 3. System Ordering with IssunSet (Medium Priority)

**Problem**: Systems should use `IssunSet` for deterministic ordering
**Pattern to Enforce**:
```rust
// ❌ FORBID in plugin build()
app.add_systems(Update, my_system);  // No ordering!

// ✅ REQUIRE in plugin build()
app.add_systems(Update, my_system.in_set(IssunSet::Logic));
```

**Scope**: `src/plugins/*/plugin.rs` (Plugin::build implementations)

**Detection**:
- Find `add_systems` calls without `.in_set(IssunSet::...)`
- Ignore test code

**Exemptions**:
- Test setup code
- Startup systems (if ordering doesn't matter)

**Estimated Effort**: Medium (AST parsing for method chains)

**Current Status**:
- AccountingPlugin: ✅ Fully compliant
- CombatPlugin: ⚠️ Pending (noted in Phase 2)

---

### 4. Vec&lt;Entity&gt; Cleanup Pattern (Medium Priority)

**Problem**: `Vec<Entity>` fields can accumulate zombie entities
**Pattern to Enforce**:
```rust
// Component with Vec<Entity>
#[derive(Component)]
pub struct Participants {
    pub entities: Vec<Entity>,  // ⚠️ Needs cleanup
}

// ✅ REQUIRE cleanup system
fn cleanup_participants(
    mut participants: Query<&mut Participants>,
    validator: Query<&RequiredComponent>,
) {
    for mut p in participants.iter_mut() {
        p.entities.retain(|e| validator.get(*e).is_ok());
    }
}
```

**Scope**: Components with `Vec<Entity>` fields

**Detection**:
- Find components with `Vec<Entity>` fields
- Verify corresponding cleanup system exists with `.retain()` pattern

**Estimated Effort**: High (requires cross-file analysis)

**Examples**:
- CombatParticipants::entities → cleanup_zombie_entities ✅
- SettlementHistory: No Vec<Entity> ✅

---

### 5. Config Resource Default Implementation (Low Priority)

**Problem**: Config resources should have sensible defaults
**Pattern to Enforce**:
```rust
// ❌ WARN if missing
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MyConfig {
    pub value: u32,
}
// Missing: impl Default for MyConfig

// ✅ REQUIRE
impl Default for MyConfig {
    fn default() -> Self {
        Self { value: 10 }
    }
}
```

**Scope**: Types marked with `#[derive(Resource)]` ending in `Config`

**Detection**:
- Find structs: `#[derive(Resource)]` + name ends with `Config`
- Verify `impl Default` exists

**Estimated Effort**: Low (AST parsing for impl blocks)

**Current Status**:
- CombatConfig: ✅ Has Default
- AccountingConfig: ✅ Has Default

---

### 6. Message Registration (Medium Priority)

**Problem**: Messages must be registered in Plugin::build()
**Pattern to Enforce**:
```rust
// All Message types must have:
app.add_message::<MyMessage>();

// All Event types must have:
app.add_event::<MyEvent>();  // If using observer events
```

**Scope**: Plugin::build() implementations

**Detection**:
- Find all `#[derive(Message)]` types
- Verify `app.add_message::<Type>()` exists in plugin
- Find all `#[derive(Event)]` types used with observers
- Verify `app.add_event::<Type>()` exists (if needed)

**Estimated Effort**: High (requires cross-file analysis)

**Current Status**: All messages properly registered ✅

---

### 7. Component Registration (Already Partially Enforced)

**Problem**: All Reflect types must be registered
**Pattern to Enforce**:
```rust
app.register_type::<MyComponent>();
```

**Status**: Partially covered by Reflect linting (#1)

**Enhancement**: Verify registration in Plugin::build() matches all Reflect types

**Estimated Effort**: Medium (cross-file analysis)

---

### 8. System Parameter Validation (Low Priority)

**Problem**: Systems should not use `&World` with mutable queries
**Pattern to Forbid**:
```rust
// ❌ FORBID: Causes borrowing conflicts
fn bad_system(
    mut query: Query<&mut Component>,
    world: &World,  // Conflicts with mutable query!
) { }

// ✅ REQUIRE: Use Query results for validation
fn good_system(
    mut query: Query<&mut Component>,
) {
    if let Ok(comp) = query.get_mut(entity) {
        // Valid
    }
}
```

**Scope**: All systems in `src/plugins/*/systems.rs`

**Detection**:
- Find functions with both `Query<&mut ...>` and `&World` parameters
- Emit error

**Estimated Effort**: Medium (AST parsing for function signatures)

---

### 9. Message vs Event Trait Usage (Medium Priority)

**Problem**: Distinguish when to use Message vs Event
**Pattern Guidelines**:
```rust
// ✅ Use Message for: Command/State notifications (buffered)
#[derive(Message, Clone, Reflect)]
pub struct SettlementRequested { }

// ✅ Use Event for: Observer extensibility points
#[derive(Event, Clone, Reflect)]
pub struct IncomeCalculationEvent { }
```

**Enforcement**: Documentation only (hard to lint semantically)

**Estimated Effort**: N/A (design guideline)

---

### 10. Test Coverage Requirements (Low Priority)

**Problem**: Systems should have corresponding tests
**Pattern to Enforce**:
- Each public system function should have at least 1 test
- Tests should cover: happy path, error cases, edge cases

**Detection**:
- Find all `pub fn` in `systems.rs`
- Verify corresponding `#[test] fn test_*` exists

**Estimated Effort**: Medium (cross-file analysis)

**Current Status**:
- CombatPlugin: 12 tests for 5 systems ✅ (>2 tests per system)
- AccountingPlugin: 9 tests for 6 systems ✅ (>1 test per system)

---

## 📊 Priority Ranking

### Must Have (P0)
1. **Entity Query Safety** - Prevents runtime panics

### Should Have (P1)
2. **System Ordering with IssunSet** - Ensures deterministic execution
3. **Message Registration** - Prevents runtime errors

### Nice to Have (P2)
4. **Vec&lt;Entity&gt; Cleanup Pattern** - Prevents memory leaks
5. **System Parameter Validation** - Prevents borrowing conflicts
6. **Config Resource Default** - Better developer experience

### Future Consideration (P3)
7. **Component Registration** - Enhancement to existing lints
8. **Test Coverage Requirements** - Quality assurance
9. **Message vs Event Guidelines** - Documentation

---

## 🛠 Implementation Strategy

### Phase 1: High-Impact, Low-Effort
1. **Entity Query Safety** (Medium effort, High impact)
   - Add to `tests/lints.rs`
   - Extend existing AST visitor pattern
   - Check for `.unwrap()` after `query.get()`

2. **Config Resource Default** (Low effort, Medium impact)
   - Simple AST check for Default impl
   - Low false positive rate

### Phase 2: Medium Priority
3. **System Ordering** (Medium effort, Medium impact)
   - Parse Plugin::build() implementations
   - Check for `.in_set()` usage

4. **System Parameter Validation** (Medium effort, Medium impact)
   - Check function signatures
   - Detect `&World` + `Query<&mut ...>`

### Phase 3: Complex Lints
5. **Vec&lt;Entity&gt; Cleanup Pattern** (High effort, Medium impact)
   - Requires cross-file analysis
   - Track component → cleanup system relationship

6. **Message Registration** (High effort, High impact)
   - Requires cross-file analysis
   - Track Message types → Plugin registration

---

## 🧪 Testing Strategy

Each lint should have:
1. **Positive test**: Code that should pass
2. **Negative test**: Code that should fail
3. **Edge case tests**: Boundary conditions

Example structure (similar to existing Reflect lints):
```rust
#[test]
fn test_entity_query_safety_detects_unwrap() {
    let violations = check_entity_query_safety("tests/fixtures/bad_unwrap");
    assert!(!violations.is_empty());
}

#[test]
fn test_entity_query_safety_accepts_if_let() {
    let violations = check_entity_query_safety("tests/fixtures/good_if_let");
    assert!(violations.is_empty());
}
```

---

## 📝 Next Steps

1. **Discuss priorities** with team
2. **Prototype Entity Query Safety lint** (highest impact)
3. **Create test fixtures** for each lint
4. **Extend `tests/lints.rs`** incrementally
5. **Document lint rules** in migration guide
6. **Add to `make preflight-bevy`** workflow

---

**End of Lint Candidates Document**
