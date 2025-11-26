# AccountingPlugin Design Document (Bevy Edition)

**Status**: Phase 2 Design
**Created**: 2025-11-26
**Updated**: 2025-11-26
**Author**: issun team
**Migration**: ISSUN v0.6 → Bevy ECS

---

## 🎯 Vision

> "Periodic financial settlements as data flow: Resources track budgets, systems process settlements, observers customize calculations."

AccountingPlugin provides a periodic financial settlement system with budget management, income/expense calculation, and fund transfers. It is a **minimal financial engine** that games can extend via Bevy's Observer pattern for custom economic behavior.

**Key Principle**: **Framework provides mechanics, games provide content**. The plugin handles settlement timing and budget tracking; games define income sources, expense calculations, and financial policies.

---

## 🧩 Problem Statement

Organizational/company management games need:

**What's Missing**:
- Periodic financial settlement system (daily/weekly/monthly)
- Multi-channel budget management (cash, research, operations, reserves)
- Income/expense calculation framework
- Budget transfer mechanics
- Settlement history tracking
- Event-driven architecture for economic triggers
- **Deterministic settlement replay capability**

**Core Challenge**: How to provide **reusable financial mechanics** while allowing **game-specific income/expense rules** and **deterministic settlement replay** without complex inheritance or trait systems?

**Use Cases**:
- Corporation simulation games (quarterly earnings)
- City builders (daily tax revenue)
- Organization management (weekly payroll)
- Research labs (grant funding periods)
- Faction operations (mission budgets)

---

## 🏗 Core Design (Bevy ECS)

### 1. Entity Structure

The accounting plugin uses the following entities and components:

```rust
/// Organization Entity (with Budget)
///
/// Composition:
/// - Organization Component (optional, for identification)
/// - BudgetLedger Component (required for accounting)
/// - SettlementHistory Component (optional, for analytics)
Entity {
    Organization,      // Optional: name, type
    BudgetLedger,     // Required: all budget channels
    SettlementHistory, // Optional: past settlements
    UniqueId,         // For replay support
}

/// Settlement Session Entity (per settlement event)
///
/// Composition:
/// - SettlementSession Component
/// - SettlementCalculation Component (income/expense breakdown)
/// - SettlementRng Component (for deterministic randomness)
Entity {
    SettlementSession,
    SettlementCalculation,
    SettlementRng,  // Per-settlement RNG (not global)
    UniqueId,       // For replay
}
```

**Design Decisions**:
- **Organizations are Entities**: Each org/company/faction is an independent Entity
- **BudgetLedger as Component**: Allows multiple budgets per world (multi-org support)
- **Settlement Sessions as Entities**: 1 settlement = 1 Entity (allows concurrent settlements, replay)
- **No Global State**: Unlike ISSUN v0.6, Bevy version supports multiple organizations
- **period as metadata**: Keep u32 period for human-readable identification (UI, logs)

### 2. Components (ECS)

#### 2.1 Budget Components

**BudgetLedger Component**: Tracks all financial channels for an organization.

**Structure**:
- 6 budget channels: Cash, Research, Operations, Reserve, Innovation, Security
- All channels use `Currency` type with saturation arithmetic

**Key Operations**:
- `can_spend(channel, amount)` - Validate sufficient funds
- `transfer(from, to, amount)` - Atomic transfer between channels
- `try_spend(channel, amount)` - Attempt to deduct funds
- `total_liquid()` - Sum of cash + pools + reserve
- `total_assets()` - Total including investment funds

**Design Decisions**:
- **Saturation arithmetic**: Prevents overflow/underflow in financial calculations
- **Atomic transfers**: Either succeeds completely or fails (no partial transfers)
- **Multi-channel design**: Allows budget allocation policies (research, ops, reserves)

#### 2.2 Settlement Components

**SettlementHistory**: Tracks past settlements for analytics and duplicate prevention.
- `last_settlement_period: u32` - Prevents duplicate settlements
- `records: Vec<SettlementRecord>` - Keeps recent 20 settlements
- Key method: `should_run_settlement()` - Checks if settlement should run

**SettlementRecord**: Historical settlement data.
- Period, day, income, expenses, net, cash_after

**SettlementSession**: Represents an active settlement calculation (spawned as entity).
- `organization: Entity` - Which org is settling
- `period: u32` - Settlement period number
- `status: SettlementStatus` - Calculating or Completed

**SettlementCalculation**: Breakdown of income/expenses for a session.
- `income_sources: Vec<IncomeSource>` - Categories + amounts
- `expense_items: Vec<ExpenseItem>` - Categories + amounts
- `total_income, total_expenses, net` - Computed totals

**SettlementRng**: Per-settlement RNG for deterministic replay.
- `seed: u64` - Derived from `hash(org_id + period)`
- ⚠️ **CRITICAL**: DO NOT use global RNG for settlement calculations!
- **Reason**: Parallel settlements, replay correctness

#### 2.3 Helper Components

**UniqueId**: Stable identifier for replay support.
- Entity IDs change between runs, so we need stable IDs
- Used for entity reference resolution during replay

**Organization**: Optional metadata component.
- `name: String` - Human-readable name
- `org_type: OrganizationType` - PlayerCompany, Faction, City, Research

### 3. Resources (Global Configuration)

**AccountingConfig**: Global configuration for the accounting system.
- `settlement_period_days: u32` - Default: 7 (weekly settlements)
- Configurable via `AccountingPlugin::with_period_days(days)`

### 4. Messages (Events)

**⚠️ CRITICAL**: Bevy 0.17 uses `Message` trait for buffered events, `Event` trait for observer events

#### 4.1 Command Messages (Requests)

**SettlementRequested**: Trigger a settlement for an organization
- `organization: Entity` - Which org to settle
- `period: u32` - Settlement period number
- Uses `Message` trait (buffered event)

**BudgetTransferRequested**: Request funds transfer between channels
- `organization, from, to, amount`
- Uses `Message` trait (buffered event)

#### 4.2 State Messages (Notifications)

**SettlementCompletedEvent**: Settlement finished notification
- `organization, period, income, expenses, net`
- Uses `Message` trait (buffered event)

**BudgetTransferredEvent**: Transfer completed notification
- `organization, from, to, amount`
- Uses `Message` trait (buffered event)

#### 4.3 Observer Events (Extensibility Points)

**IncomeCalculationEvent**: Triggered during income calculation
- `settlement_entity, organization, period, sources`
- Uses `Event` trait (observer pattern)
- **Purpose**: Observers can add custom income sources

**ExpenseCalculationEvent**: Triggered during expense calculation
- `settlement_entity, organization, period, items`
- Uses `Event` trait (observer pattern)
- **Purpose**: Observers can add custom expense items

---

## 🎬 Replay System Design

### Replay Architecture

**Challenge**: Settlements involve:
- Time-based triggering (DayChanged events)
- Income/expense calculations (may use RNG)
- Budget updates
- Multiple organizations settling concurrently

**Solution**:
1. **Per-Settlement RNG**: Each SettlementSession has its own `SettlementRng` component
2. **Command Message Recording**: Record only `SettlementRequested` messages (not `DayChanged`)
3. **Frame-Based Timing**: Record frame number, not wall-clock time
4. **UniqueId Mapping**: Use stable IDs for entity references

#### Replay Components

**SettlementRecorder**: Attached to an organization to record settlements.
- `recorded_commands: Vec<RecordedSettlement>` - History of settlement triggers

**RecordedSettlement**: Recorded settlement data.
- `frame: u32` - Frame number when settlement triggered
- `period: u32` - Settlement period
- `organization_id: String` - UniqueId (stable across runs, not Entity)
- `rng_seed: u64` - Seed for deterministic replay

#### Replay Flow

```
Recording Mode:
  1. DayChanged → Check Period → Settlement Trigger
  2. Generate RNG seed: hash(org_id + period)
  3. Create SettlementSession with SettlementRng(seed)
  4. Record: { frame, period, org_id, seed }
  5. Calculate Income/Expenses (using SettlementRng)
  6. Apply Settlement

Replay Mode:
  1. Playback Frame X → Load RecordedSettlement
  2. Resolve org_id → Entity (via UniqueId query)
  3. Create SettlementSession with SettlementRng(recorded_seed)
  4. Calculate Income/Expenses (deterministic with same seed)
  5. Verify: Settlement result matches recorded result
```

**Key Insight**: Record only the settlement trigger command + RNG seed, NOT the entire calculation results. This keeps replay data compact and allows verification of calculation determinism.

### Replay Requirements

1. **Income/Expense Calculation must be deterministic**
   - Use SettlementRng component (NOT global RNG)
   - Avoid system time, random network calls
   - Sort queries consistently (e.g., by UniqueId)

2. **Entity Reference Resolution**
   - Record UniqueId (String), not Entity
   - Resolve UniqueId → Entity during replay

3. **Observer Order Matters**
   - Income/expense observers must run in deterministic order
   - Use `.chain()` or system ordering

### Replay Limitations

- **Network calls during settlement**: Not supported (use pre-fetched data)
- **User input during settlement**: Not supported (settlements are automatic)
- **Global RNG usage**: Will cause desync

---

## 🔄 System Flow

### System Execution Order

**IssunSet::Input**
- `handle_day_changed` - Detect period boundaries, trigger settlements

**IssunSet::Logic** (chained order)
1. `start_settlement_sessions` - Spawn settlement entities
2. `calculate_income` - Trigger observer events for income customization
3. `calculate_expenses` - Trigger observer events for expense customization
4. `finalize_settlements` - Apply to BudgetLedger, publish completed events
5. `handle_budget_transfers` - Process transfer requests (independent)

**IssunSet::PostLogic**
- `cleanup_settlement_sessions` - Despawn completed sessions

### Settlement Flow (Detailed)

```
1. handle_day_changed (IssunSet::Input)
   ├─ Read DayChanged messages
   ├─ Query: Organizations with (BudgetLedger, SettlementHistory)
   ├─ Check: history.should_run_settlement(current_day, period_days)
   └─ Write: SettlementRequested message (if period boundary)

2. start_settlement_sessions (IssunSet::Logic)
   ├─ Read: SettlementRequested messages
   ├─ Generate RNG seed: hash(org_id + period)
   ├─ Spawn: SettlementSession Entity
   │   ├─ SettlementSession component
   │   ├─ SettlementCalculation component (empty)
   │   ├─ SettlementRng component (with seed)
   │   └─ UniqueId component
   └─ Record: (if recording enabled)

3. calculate_income (IssunSet::Logic, OBSERVERS CUSTOMIZE THIS)
   ├─ Query: SettlementSessions with (SettlementSession, SettlementCalculation)
   ├─ Trigger: IncomeCalculationEvent (for observers)
   ├─ Observers respond: Add IncomeSource to calculation
   └─ Sum: calculation.total_income

4. calculate_expenses (IssunSet::Logic, OBSERVERS CUSTOMIZE THIS)
   ├─ Query: SettlementSessions with (SettlementSession, SettlementCalculation)
   ├─ Trigger: IncomeCalculationEvent (for observers)
   ├─ Observers respond: Add ExpenseItem to calculation
   └─ Sum: calculation.total_expenses

5. finalize_settlements (IssunSet::Logic)
   ├─ Query: SettlementSessions with (SettlementCalculation, status=Calculating)
   ├─ Calculate: net = total_income - total_expenses
   ├─ Apply: Update BudgetLedger.cash (saturating_add)
   ├─ Update: SettlementHistory (record settlement)
   ├─ Write: SettlementCompletedEvent message
   └─ Update: status = Completed

6. cleanup_settlement_sessions (IssunSet::PostLogic)
   ├─ Query: SettlementSessions with (status=Completed)
   └─ Despawn: Session entities (after 1 frame delay)
```

### Budget Transfer Flow

```
1. handle_budget_transfers (IssunSet::Logic)
   ├─ Read: BudgetTransferRequested messages
   ├─ Query: BudgetLedger for organization
   ├─ Validate: ledger.can_spend(from, amount)
   ├─ Transfer: ledger.transfer(from, to, amount)
   ├─ Write: BudgetTransferredEvent message (if success)
   └─ Warn: If validation failed
```

---

## 🔌 Customization Points (Observer Pattern)

### 1. Custom Income Calculation

**Use Case**: Add territory tax, weapon sales, faction operations as income.

**Observer Signature**:
```rust
fn custom_income_observer(
    trigger: Trigger<IncomeCalculationEvent>,
    mut sessions: Query<&mut SettlementCalculation>,
    // ... custom queries (territories, sales, etc.)
)
```

**How It Works**:
1. System triggers `IncomeCalculationEvent` with `settlement_entity`
2. Observer receives event, queries settlement calculation
3. Observer adds custom `IncomeSource` items to `calc.income_sources`
4. Observer recalculates `calc.total_income`

**Example Income Sources**:
- Territory Tax: `population * tax_rate`
- Weapon Sales: Sum of sales records for period
- Faction Operations: Mission rewards, contract payments

### 2. Custom Expense Calculation

**Use Case**: Add faction deployment costs, research expenses, maintenance.

**Observer Signature**:
```rust
fn custom_expense_observer(
    trigger: Trigger<ExpenseCalculationEvent>,
    mut sessions: Query<&mut SettlementCalculation>,
    // ... custom queries (factions, research, etc.)
)
```

**How It Works**:
1. System triggers `ExpenseCalculationEvent` with `settlement_entity`
2. Observer receives event, queries settlement calculation
3. Observer adds custom `ExpenseItem` items to `calc.expense_items`
4. Observer recalculates `calc.total_expenses`

**Example Expense Items**:
- Faction Deployments: Active faction deployment costs
- Research Projects: R&D budget per period
- Maintenance: Infrastructure upkeep costs

### 3. Post-Settlement Triggers

**Use Case**: Trigger achievements, unlock features, update KPIs.

**Observer Signature**:
```rust
fn settlement_achievement_observer(
    trigger: Trigger<SettlementCompletedEvent>,
    // ... mutations (commands, resources)
)
```

**How It Works**:
1. System publishes `SettlementCompletedEvent` after settlement finalized
2. Observer receives event with final income/expense/net values
3. Observer can trigger achievements, unlock features, update dashboards

**Example Triggers**:
- Achievement: Net profit > 10,000
- Bankruptcy Warning: Net profit < 0
- KPI Dashboard: Record settlement history

### 4. Reserve Fund Allocation

**Use Case**: Automatically allocate percentage of profits to reserve.

**How It Works**:
1. Listen to `SettlementCompletedEvent`
2. If net profit > 0, transfer X% from cash to reserve channel
3. Uses `BudgetLedger::transfer()` method

**Example Policy**: 20% of profits to reserve fund

---

## 📊 Entity Lifecycle

### Settlement Session Lifecycle

```
Frame N:   DayChanged (period boundary detected)
           ↓
Frame N:   SettlementRequested message written
           ↓
Frame N+1: SettlementSession Entity spawned
           - SettlementSession component
           - SettlementCalculation component (empty)
           - SettlementRng component
           - status = Calculating
           ↓
Frame N+1: Income calculation (observers add sources)
           ↓
Frame N+1: Expense calculation (observers add items)
           ↓
Frame N+1: Settlement finalized
           - BudgetLedger updated
           - SettlementHistory updated
           - SettlementCompletedEvent written
           - status = Completed
           ↓
Frame N+2: Session Entity despawned (cleanup)
```

### Entity Cleanup & Zombie Prevention

**Problem**: SettlementSession references organization Entity. What if org is deleted?

**Solution Strategy**:

1. **Validate Entity Existence**:
   - Use `if let Ok(...)` pattern for all `query.get()` / `query.get_mut()` calls
   - If entity not found, log warning and despawn session

2. **Graceful Degradation**:
   - Missing BudgetLedger: Warn and skip settlement application
   - Missing SettlementHistory: Warn but continue (history is optional)
   - Deleted organization: Despawn session entity immediately

3. **Never Panic in Systems**:
   - ❌ `.unwrap()` on entity queries → ✅ `if let Ok(...)`
   - ❌ `.expect()` on component access → ✅ Pattern matching
   - ❌ Assume entity exists → ✅ Validate before use

**Entity Validation Checklist**:
- [ ] `query.get()` / `query.get_mut()` wrapped in `if let Ok(...)`
- [ ] `.unwrap()` never used (except in tests with known entities)
- [ ] Warn/log when entity validation fails
- [ ] Despawn orphaned session entities

---

## ✅ Success Criteria

### Functional Requirements

- [ ] **Periodic Settlement**: Settlements run automatically at period boundaries
- [ ] **Multi-Channel Budgets**: Cash, Research, Ops, Reserve, Innovation, Security
- [ ] **Budget Transfers**: Funds can move between channels
- [ ] **Settlement History**: Past 20 settlements tracked
- [ ] **Observer Extensibility**: Income/expense customization via observers
- [ ] **Deterministic Replay**: Settlements produce same results with same seed
- [ ] **Multi-Organization Support**: Multiple orgs can settle independently
- [ ] **Saturation Arithmetic**: No overflow/underflow in financial calculations

### Non-Functional Requirements

- [ ] **Zero Allocation**: Settlement calculations use stack-allocated data
- [ ] **Parallel Settlement**: Multiple orgs can settle in same frame
- [ ] **No Global State**: All state in components (thread-safe)
- [ ] **Reflection Support**: All types have `#[derive(Reflect)]`
- [ ] **Entity Lifecycle Safety**: No panics from deleted entities

### Testing Requirements

- [ ] **UT: Basic Settlement**: Simple income/expense calculation
- [ ] **UT: Budget Transfer**: Valid and invalid transfers
- [ ] **UT: Settlement History**: Recording and querying
- [ ] **UT: Duplicate Prevention**: Same period doesn't settle twice
- [ ] **UT: Multi-Org**: Two orgs settle independently
- [ ] **UT: Observer Customization**: Income/expense observers work
- [ ] **UT: Deleted Entity Handling**: No panics when org deleted
- [ ] **UT: Replay**: Same seed produces same results
- [ ] **UT: Saturation**: Large income doesn't overflow

---

## 🎯 Design Philosophy

### 1. Data-Oriented Design

**Components are data, systems are logic.**

- BudgetLedger is pure data (6 currency fields)
- Settlement calculations happen in systems
- No methods on components except helpers

### 2. Composition Over Inheritance

**No trait hierarchy, just components.**

```rust
// ❌ ISSUN v0.6 approach (trait inheritance)
trait AccountingHook {
    async fn calculate_income(...) -> Currency;
    async fn calculate_expenses(...) -> Currency;
}

// ✅ Bevy approach (observer pattern)
app.observe(my_income_calculator);
app.observe(my_expense_calculator);
```

### 3. Explicit State Machines

**Settlement status as enum, not booleans.**

```rust
// ❌ Bad (boolean soup)
struct Settlement {
    calculating: bool,
    completed: bool,
    failed: bool,
}

// ✅ Good (explicit state)
enum SettlementStatus {
    Calculating,
    Completed,
}
```

### 4. Saturation Arithmetic

**Prevent overflow/underflow in financial calculations.**

```rust
// ❌ Bad (can overflow)
ledger.cash = ledger.cash + net;

// ✅ Good (saturation)
ledger.cash = ledger.cash.saturating_add(net);
```

### 5. Per-Entity RNG

**No global RNG for deterministic replay.**

```rust
// ❌ Bad (global RNG, replay breaks)
let random_bonus = global_rng.gen_range(0..100);

// ✅ Good (per-settlement RNG)
let random_bonus = settlement_rng.gen_range(0..100);
```

---

## 🔮 Future Extensions

### Phase 2+

**Not in Phase 2, but designed for easy addition:**

1. **Tax System**
   - Progressive tax rates
   - Tax deductions
   - Tax events (audits, refunds)

2. **Loan System**
   - Borrow from reserve
   - Interest payments
   - Loan repayment schedules

3. **Investment System**
   - Stock market integration
   - ROI calculations
   - Portfolio management

4. **Budget Forecasting**
   - Predict next settlement
   - Cash flow projections
   - Budget alerts

5. **Accounting Reports**
   - Balance sheets
   - Income statements
   - Cash flow statements

6. **Multi-Currency Support**
   - Currency exchange rates
   - Foreign transactions
   - Hedging

7. **Fiscal Policies**
   - Austerity measures (expense reduction)
   - Stimulus spending (budget injections)
   - Subsidy programs

---

## 📚 Related Plugins

### Dependencies

- **TimePlugin** (required): Provides `DayChanged` events for settlement timing
- **EconomyPlugin** (required): Provides `Currency` type

### Integration Points

- **FactionPlugin**: Faction operations generate expenses
- **TerritoryPlugin**: Territory tax generates income
- **ResearchPlugin**: Research projects generate expenses
- **MarketPlugin**: Sales transactions generate income

---

## 🧪 Implementation Strategy

### Phase 2: Core Mechanics (Design)

**Deliverables**:
- [x] Entity structure defined
- [x] Component design complete
- [x] Message definitions complete
- [x] System flow documented
- [x] Observer pattern defined
- [x] Replay system designed
- [x] Migration notes written

### Phase 2: Implementation (Next Steps)

**Tasks**:

1. **Create files** (1h):
   ```
   crates/issun-bevy/src/plugins/accounting/
   ├── mod.rs
   ├── components.rs   (BudgetLedger, SettlementHistory, SettlementSession, etc.)
   ├── events.rs       (SettlementRequested, BudgetTransferRequested, etc.)
   ├── systems.rs      (settlement flow systems)
   ├── plugin.rs       (Plugin impl)
   └── tests.rs        (UTs)
   ```

2. **Implement Currency type** (2h):
   - Create `Currency` struct in economy plugin (if not exists)
   - Saturation arithmetic
   - Operator overloading

3. **Implement BudgetLedger** (2h):
   - Component definition
   - Channel operations
   - Transfer logic

4. **Implement Settlement Systems** (6h):
   - handle_day_changed
   - start_settlement_sessions
   - calculate_income (with observer support)
   - calculate_expenses (with observer support)
   - finalize_settlements
   - cleanup_settlement_sessions

5. **Implement Budget Transfer** (2h):
   - handle_budget_transfers system
   - Validation logic

6. **Write Tests** (4h):
   - Basic settlement flow
   - Budget transfers
   - Multi-org settlements
   - Observer customization
   - Deleted entity handling
   - Replay determinism

7. **Run CHECK STEP** (5min):
   ```bash
   make preflight-bevy
   ```

**Total Estimate**: 17 hours (2-3 days)

### Phase 3+: Extensions

- Replay recording/playback
- Settlement analytics
- Budget forecasting
- Tax system
- Loan system

---

## 📋 Migration Notes (ISSUN v0.6 → Bevy)

### Key Changes

| ISSUN v0.6 | Bevy ECS | Reason |
|------------|----------|--------|
| `AccountingState` (global) | `BudgetLedger` (component) | Multi-org support |
| `AccountingHook` (trait) | Observer pattern | Bevy's extension mechanism |
| `#[resource]` | `#[derive(Resource)]` | Bevy's resource system |
| `#[state]` | `#[derive(Component)]` | ECS state management |
| `async fn` | Sync systems | Bevy's synchronous ECS |
| `EventBus` | `MessageWriter` / `MessageReader` | Bevy 0.17 messaging |
| String IDs | Entity IDs | Native ECS references |
| Global settlement | Settlement sessions | Parallel processing |
| `Currency` (economy plugin) | `Currency` (shared type) | Type reuse |

### String ID → Entity Migration

| ISSUN v0.6 | Bevy ECS |
|------------|----------|
| Global `AccountingState` | Per-org `BudgetLedger` component |
| No organization ID in events | Explicit `organization: Entity` |
| Single organization only | Multi-org support via ECS |

**Impact**: Settlements can now run for multiple organizations independently.

### Hook → Observer Migration

| ISSUN v0.6 | Bevy ECS |
|------------|----------|
| `#[async_trait] impl AccountingHook` | `fn income_observer(trigger: Trigger<...>)` |
| Trait methods (inheritance) | Observer functions (composition) |
| `async fn` (async runtime) | Sync systems (Bevy ECS) |

**Impact**: Simpler to extend, no async complexity, better performance.

### Reflect Requirements

**All Bevy types must have**:
- Components/Resources: `#[derive(Reflect)]` + `#[reflect(Component/Resource)]`
- Messages: `#[derive(Reflect)]` + `#[reflect(opaque)]` (NOT `#[reflect(Message)]`)
- Events: `#[derive(Reflect)]` only (NOT `#[reflect(Event)]`)
- Plugin registration: `app.register_type::<T>()`

**Enforcement**: Static linting via `tests/lints.rs` in `make preflight-bevy`

---

## 🎬 Implementation Checklist

### Component Implementation

- [ ] `BudgetLedger` component with all channels
- [ ] `SettlementHistory` component
- [ ] `SettlementSession` component
- [ ] `SettlementCalculation` component
- [ ] `SettlementRng` component
- [ ] `UniqueId` component
- [ ] All components have `#[derive(Reflect)]` + `#[reflect(Component)]`

### Message Implementation

- [ ] `SettlementRequested` message
- [ ] `BudgetTransferRequested` message
- [ ] `SettlementCompletedEvent` message
- [ ] `BudgetTransferredEvent` message
- [ ] `IncomeCalculationEvent` event (observer)
- [ ] `ExpenseCalculationEvent` event (observer)
- [ ] All messages have `#[derive(Reflect)]` + `#[reflect(Message)]`

### System Implementation

- [ ] `handle_day_changed` system
- [ ] `start_settlement_sessions` system
- [ ] `calculate_income` system
- [ ] `calculate_expenses` system
- [ ] `finalize_settlements` system
- [ ] `handle_budget_transfers` system
- [ ] `cleanup_settlement_sessions` system
- [ ] All systems use `IssunSet` for ordering
- [ ] All entity accesses use `if let Ok(...)`

### Plugin Implementation

- [ ] `AccountingPlugin` struct
- [ ] Plugin `build()` method
- [ ] `app.register_type::<T>()` for all types
- [ ] `app.add_message::<M>()` for all messages
- [ ] `app.add_event::<E>()` for observer events
- [ ] Systems added with correct ordering

### Testing Implementation

- [ ] Basic settlement test
- [ ] Budget transfer test (valid)
- [ ] Budget transfer test (invalid)
- [ ] Multi-org settlement test
- [ ] Observer customization test
- [ ] Deleted entity test (no panic)
- [ ] Replay determinism test
- [ ] Saturation arithmetic test
- [ ] `make preflight-bevy` passes

---

## 📖 Usage Summary

### Basic Setup

1. **Add Plugin**: `app.add_plugins(AccountingPlugin::default())`
2. **Spawn Organization**:
   - Components: `Organization`, `BudgetLedger`, `SettlementHistory`, `UniqueId`
3. **Settlements run automatically** based on `AccountingConfig::settlement_period_days`

### Custom Income/Expense

1. **Register Observers**: `app.observe(custom_income_observer)`
2. **Income Observer**: Respond to `IncomeCalculationEvent`, add `IncomeSource` items
3. **Expense Observer**: Respond to `ExpenseCalculationEvent`, add `ExpenseItem` items

### Manual Settlement Trigger

- Write `SettlementRequested` message with `organization` entity and `period`
- Useful for UI buttons, debug commands, special events

### Budget Transfers

- Write `BudgetTransferRequested` message with `organization`, `from`, `to`, `amount`
- System validates funds and publishes `BudgetTransferredEvent` on success

---

**End of Design Document**
