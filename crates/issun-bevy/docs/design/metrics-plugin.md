# MetricsPlugin Design Document (Bevy Edition)

**Status**: Phase 2 Design
**Created**: 2025-11-26
**Updated**: 2025-11-26
**Author**: issun team
**Migration**: ISSUN v0.6 → Bevy ECS

---

## 🎯 Vision

> "Game metrics as observable data streams: Resources track measurements, systems process aggregations, observers customize analytics."

MetricsPlugin provides a comprehensive metrics collection, aggregation, and reporting system. It is a **minimal observability engine** that games can extend via Bevy's Observer pattern for custom analytics and monitoring.

**Key Principle**: **Framework provides mechanics, games provide insights**. The plugin handles metric storage, aggregation, and reporting; games define what to measure and how to react to measurements.

---

## 🧩 Problem Statement

Modern games need robust observability:

**What's Missing**:
- Real-time metrics collection (FPS, player stats, economy metrics)
- Time-series data storage with memory efficiency
- Statistical aggregation (percentiles, averages, rates)
- Periodic snapshots for state capture
- Report generation for analytics
- Event-driven architecture for alerts/monitoring
- **Extensibility for game-specific metrics and reactions**

**Core Challenge**: How to provide **reusable metrics infrastructure** while allowing **game-specific measurement strategies** and **custom analytics** without complex trait systems or performance overhead?

**Use Cases**:
- Performance monitoring (FPS, frame time, memory usage)
- Player analytics (actions per minute, damage dealt, resources collected)
- Economy tracking (currency flow, market prices, transaction volumes)
- AI behavior metrics (decision times, pathfinding performance)
- Server health monitoring (connection count, latency, throughput)

---

## 🏗 Core Design (Bevy ECS)

### 1. Architecture Overview

Unlike AccountingPlugin (which uses Entity-based multi-org design), MetricsPlugin uses a **global Resource-based design**:

```rust
/// Global Metrics Architecture
World {
    Resources {
        MetricsRegistry,   // Single global registry
        MetricsConfig,     // Global configuration
        MetricsHistory,    // Optional: historical snapshots
    },
    Messages {
        // Command Messages (buffered)
        DefineMetricRequested,
        RecordMetricRequested,
        CreateSnapshotRequested,
        GenerateReportRequested,
        RemoveMetricRequested,
        ClearMetricsRequested,

        // State Messages (notifications)
        MetricDefined,
        MetricRecorded,
        SnapshotCreated,
        ReportGenerated,
        MetricRemoved,
        MetricsCleared,
    },
    Events {
        // Observer Events (extensibility)
        MetricRecordedEvent,
        SnapshotCreatedEvent,
        ReportGeneratedEvent,
    }
}
```

**Design Decisions**:
- **Global Registry**: Single `MetricsRegistry` Resource (no per-entity metrics)
- **No Entities**: Metrics are not ECS entities (pure data in Resource)
- **Message-Driven**: All operations via Messages (request/notification pattern)
- **Observer Extensibility**: Game-specific analytics via Bevy Observers
- **Stateless Aggregation**: Aggregation functions are pure (no mutation)

**Rationale**: Metrics are inherently global (FPS, memory usage, etc.) and don't need entity-based design. This simplifies the architecture and improves performance.

### 2. Resources (Global State)

#### 2.1 MetricsRegistry Resource

**Purpose**: Central storage for metric definitions and time-series values.

**Structure**:
```rust
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MetricsRegistry {
    /// Metric definitions by ID
    definitions: HashMap<MetricId, MetricDefinition>,

    /// Time-series values (windowed storage)
    values: HashMap<MetricId, VecDeque<MetricValue>>,

    /// Configuration
    config: MetricsConfig,
}
```

**Key Operations**:
- `define(definition: MetricDefinition)` - Register a metric
- `record(value: MetricValue) -> Result<()>` - Record a measurement
- `remove(metric_id: &MetricId)` - Remove a metric
- `clear()` - Clear all metrics
- `aggregate(metric_id, aggregation, start, end) -> Option<AggregatedMetric>` - Compute aggregation
- `all_values() -> Vec<(MetricId, Vec<MetricValue>)>` - Get all values (for snapshots)
- `get_definition(metric_id) -> Option<&MetricDefinition>` - Get metric definition

**Memory Management**:
- Uses `VecDeque` with configurable `max_values_per_metric` limit
- Oldest values automatically evicted when limit reached (FIFO)
- Default: 1000 values per metric

**Design Decisions**:
- **Windowed Storage**: Memory-efficient for long-running games
- **HashMap Lookup**: O(1) access by MetricId
- **VecDeque**: Efficient push_back/pop_front for FIFO
- **Clone-on-Read**: Values cloned for safety (no shared mutable state)

#### 2.2 MetricsConfig Resource

**Purpose**: Global configuration for metrics system.

**Structure**:
```rust
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct MetricsConfig {
    /// Max values stored per metric (default: 1000)
    pub max_values_per_metric: usize,

    /// Enable periodic snapshots (default: false)
    pub enable_periodic_snapshots: bool,

    /// Snapshot period in days (default: 1)
    pub snapshot_period: u32,

    /// Enable auto-generated reports (default: false)
    pub enable_auto_report: bool,

    /// Report period in days (default: 7)
    pub report_period: u32,
}
```

**Configurable Via**:
```rust
MetricsPlugin::default()
    .with_config(MetricsConfig {
        max_values_per_metric: 2000,
        enable_periodic_snapshots: true,
        snapshot_period: 1,
        enable_auto_report: true,
        report_period: 7,
    })
```

#### 2.3 MetricsHistory Resource (Optional)

**Purpose**: Store historical snapshots and reports for analytics.

**Structure**:
```rust
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct MetricsHistory {
    /// Recent snapshots (max 100)
    snapshots: VecDeque<MetricSnapshot>,

    /// Recent reports (max 50)
    reports: VecDeque<MetricReport>,
}
```

**Design Decision**: Optional Resource - only created if user needs historical tracking.

### 3. Data Types (Core Structures)

#### 3.1 Metric Identity

```rust
/// Unique metric identifier
#[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect)]
#[reflect(opaque)]
pub struct MetricId(pub String);

/// Metric type classification
#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(opaque)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,

    /// Point-in-time value (can go up or down)
    Gauge,

    /// Distribution of values (for percentiles)
    Histogram,
}
```

#### 3.2 Metric Definition

```rust
/// Metric metadata and configuration
#[derive(Debug, Clone, Reflect)]
#[reflect(opaque)]
pub struct MetricDefinition {
    pub metric_id: MetricId,
    pub metric_type: MetricType,
    pub description: String,

    /// Semantic tags (e.g., "performance", "economy")
    pub tags: Vec<String>,

    /// Key-value labels (e.g., {"region": "NA", "server": "prod"})
    pub labels: HashMap<String, String>,
}
```

#### 3.3 Metric Value

```rust
/// Single metric measurement
#[derive(Debug, Clone, Reflect)]
#[reflect(opaque)]
pub struct MetricValue {
    pub metric_id: MetricId,
    pub value: f64,
    pub timestamp: u64,  // Unix timestamp in seconds

    /// Optional metadata (JSON)
    pub metadata: serde_json::Value,
}
```

#### 3.4 Aggregation Types

```rust
/// Statistical aggregation methods
#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(opaque)]
pub enum AggregationType {
    Sum,        // Total sum
    Count,      // Number of values
    Average,    // Mean value
    Min,        // Minimum value
    Max,        // Maximum value
    P50,        // 50th percentile (median)
    P95,        // 95th percentile
    P99,        // 99th percentile
    Last,       // Most recent value
    Rate,       // Count per second
}

/// Result of aggregation
#[derive(Debug, Clone, Reflect)]
#[reflect(opaque)]
pub struct AggregatedMetric {
    pub metric_id: MetricId,
    pub aggregation_type: AggregationType,
    pub value: f64,
    pub period_start: u64,
    pub period_end: u64,
    pub sample_count: usize,
}
```

#### 3.5 Snapshots and Reports

```rust
/// Point-in-time snapshot of all metrics
#[derive(Debug, Clone, Reflect)]
#[reflect(opaque)]
pub struct MetricSnapshot {
    pub timestamp: u64,
    pub values: HashMap<MetricId, Vec<MetricValue>>,
    pub label: Option<String>,  // e.g., "daily_snapshot_day_7"
}

/// Aggregated report for a time period
#[derive(Debug, Clone, Reflect)]
#[reflect(opaque)]
pub struct MetricReport {
    pub period_start: u64,
    pub period_end: u64,
    pub aggregated_metrics: Vec<AggregatedMetric>,
    pub label: Option<String>,  // e.g., "weekly_report_week_42"
}
```

### 4. Messages (Events)

**⚠️ CRITICAL**: Bevy 0.17 uses `Message` trait for buffered events, `Event` trait for observer events

#### 4.1 Command Messages (Requests)

**DefineMetricRequested**: Register a new metric
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct DefineMetricRequested {
    pub definition: MetricDefinition,
}
```

**RecordMetricRequested**: Record a measurement
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct RecordMetricRequested {
    pub value: MetricValue,
}
```

**CreateSnapshotRequested**: Create snapshot of all metrics
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct CreateSnapshotRequested {
    pub timestamp: u64,
    pub label: Option<String>,
}
```

**GenerateReportRequested**: Generate aggregated report
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct GenerateReportRequested {
    pub period_start: u64,
    pub period_end: u64,
    pub metric_ids: Vec<MetricId>,
    pub aggregations: Vec<AggregationType>,
    pub label: Option<String>,
}
```

**RemoveMetricRequested**: Remove a metric
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct RemoveMetricRequested {
    pub metric_id: MetricId,
}
```

**ClearMetricsRequested**: Clear all metrics
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ClearMetricsRequested;
```

#### 4.2 State Messages (Notifications)

**MetricDefined**: Metric successfully defined
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricDefined {
    pub definition: MetricDefinition,
}
```

**MetricRecorded**: Metric value successfully recorded
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricRecorded {
    pub value: MetricValue,
}
```

**SnapshotCreated**: Snapshot successfully created
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SnapshotCreated {
    pub snapshot: MetricSnapshot,
}
```

**ReportGenerated**: Report successfully generated
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ReportGenerated {
    pub report: MetricReport,
}
```

**MetricRemoved**: Metric removed
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricRemoved {
    pub metric_id: MetricId,
}
```

**MetricsCleared**: All metrics cleared
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricsCleared {
    pub timestamp: u64,
}
```

#### 4.3 Observer Events (Extensibility Points)

**MetricRecordedEvent**: Triggered after metric recorded (for alerts, monitoring)
```rust
#[derive(Event, Clone, Debug)]
pub struct MetricRecordedEvent {
    pub value: MetricValue,
}
```
**Purpose**: Observers can:
- Trigger alerts on thresholds (e.g., FPS < 30)
- Send to external monitoring (Prometheus, Datadog)
- Update dashboards

**SnapshotCreatedEvent**: Triggered after snapshot created (for storage)
```rust
#[derive(Event, Clone, Debug)]
pub struct SnapshotCreatedEvent {
    pub snapshot: MetricSnapshot,
}
```
**Purpose**: Observers can:
- Archive snapshots to disk/cloud
- Compress and export historical data

**ReportGeneratedEvent**: Triggered after report generated (for distribution)
```rust
#[derive(Event, Clone, Debug)]
pub struct ReportGeneratedEvent {
    pub report: MetricReport,
}
```
**Purpose**: Observers can:
- Email reports to stakeholders
- Export to BI tools
- Trigger achievements based on performance

---

## 🔄 System Flow

### System Execution Order

**IssunSet::Input**
- `periodic_snapshot_system` - Trigger snapshots at configured periods (if enabled)
- `periodic_report_system` - Trigger reports at configured periods (if enabled)

**IssunSet::Logic** (chained order)
1. `process_define_requests` - Register metrics
2. `process_record_requests` - Record values
3. `process_snapshot_requests` - Create snapshots
4. `process_report_requests` - Generate reports
5. `process_remove_requests` - Remove metrics
6. `process_clear_requests` - Clear registry

**IssunSet::PostLogic**
- `trigger_observer_events` - Fire observer events for customization

### Detailed System Flow

#### 1. Define Metric Flow
```
User/System → Write DefineMetricRequested message
             ↓
process_define_requests (IssunSet::Logic)
├─ Read: DefineMetricRequested messages
├─ Mutate: MetricsRegistry.define()
├─ Write: MetricDefined message
└─ (No observer event for definitions)
```

#### 2. Record Metric Flow
```
User/System → Write RecordMetricRequested message
             ↓
process_record_requests (IssunSet::Logic)
├─ Read: RecordMetricRequested messages
├─ Mutate: MetricsRegistry.record()
│   ├─ Validate: metric exists
│   ├─ Push: value to VecDeque
│   └─ Evict: oldest value if over limit
├─ Write: MetricRecorded message
└─ Queue: MetricRecordedEvent for observers
             ↓
trigger_observer_events (IssunSet::PostLogic)
└─ Trigger: MetricRecordedEvent
             ↓
Observers (user-defined)
└─ Custom reactions (alerts, monitoring, etc.)
```

#### 3. Snapshot Flow
```
periodic_snapshot_system (IssunSet::Input)
├─ Check: config.enable_periodic_snapshots
├─ Check: DayChanged % config.snapshot_period == 0
└─ Write: CreateSnapshotRequested
             ↓
process_snapshot_requests (IssunSet::Logic)
├─ Read: CreateSnapshotRequested messages
├─ Query: MetricsRegistry.all_values()
├─ Create: MetricSnapshot with all current values
├─ Optionally: Store in MetricsHistory
├─ Write: SnapshotCreated message
└─ Queue: SnapshotCreatedEvent for observers
             ↓
trigger_observer_events (IssunSet::PostLogic)
└─ Trigger: SnapshotCreatedEvent
             ↓
Observers (user-defined)
└─ Archive snapshots to storage
```

#### 4. Report Flow
```
periodic_report_system (IssunSet::Input)
├─ Check: config.enable_auto_report
├─ Check: DayChanged % config.report_period == 0
├─ Determine: period_start, period_end
└─ Write: GenerateReportRequested
             ↓
process_report_requests (IssunSet::Logic)
├─ Read: GenerateReportRequested messages
├─ For each metric_id + aggregation:
│   └─ Call: registry.aggregate(id, agg, start, end)
├─ Create: MetricReport with aggregations
├─ Optionally: Store in MetricsHistory
├─ Write: ReportGenerated message
└─ Queue: ReportGeneratedEvent for observers
             ↓
trigger_observer_events (IssunSet::PostLogic)
└─ Trigger: ReportGeneratedEvent
             ↓
Observers (user-defined)
└─ Email reports, export to BI, trigger achievements
```

---

## 🔌 Customization Points (Observer Pattern)

### 1. Custom Metric Recording Reactions

**Use Case**: Send metrics to external monitoring, trigger alerts.

**Observer Signature**:
```rust
fn metric_alert_observer(
    trigger: Trigger<MetricRecordedEvent>,
    mut commands: Commands,
    registry: Res<MetricsRegistry>,
) {
    let value = &trigger.event().value;

    // Example: FPS alert
    if value.metric_id == MetricId("fps".into()) && value.value < 30.0 {
        warn!("FPS dropped below 30: {}", value.value);
        // Trigger alert UI
        commands.write_message(ShowAlertRequested {
            message: format!("Performance warning: {} FPS", value.value),
        });
    }
}
```

**How to Register**:
```rust
app.observe(metric_alert_observer);
```

### 2. Snapshot Archival

**Use Case**: Save snapshots to disk for historical analysis.

**Observer Signature**:
```rust
fn snapshot_archival_observer(
    trigger: Trigger<SnapshotCreatedEvent>,
) {
    let snapshot = &trigger.event().snapshot;

    // Serialize and save to disk
    let json = serde_json::to_string_pretty(snapshot).unwrap();
    let filename = format!("snapshots/snapshot_{}.json", snapshot.timestamp);
    std::fs::write(filename, json).ok();
}
```

### 3. Report Distribution

**Use Case**: Email weekly reports to stakeholders.

**Observer Signature**:
```rust
fn report_email_observer(
    trigger: Trigger<ReportGeneratedEvent>,
    email_service: Res<EmailService>,
) {
    let report = &trigger.event().report;

    // Format report as HTML
    let html = format_report_as_html(report);

    // Send email
    email_service.send(
        "stakeholders@company.com",
        "Weekly Metrics Report",
        html,
    );
}
```

### 4. Achievement Triggers

**Use Case**: Unlock achievements based on performance metrics.

**Observer Signature**:
```rust
fn achievement_observer(
    trigger: Trigger<ReportGeneratedEvent>,
    mut commands: Commands,
) {
    let report = &trigger.event().report;

    // Check for "Speedrunner" achievement (average FPS > 60)
    if let Some(avg_fps) = report.aggregated_metrics.iter().find(|m| {
        m.metric_id == MetricId("fps".into())
        && m.aggregation_type == AggregationType::Average
    }) {
        if avg_fps.value > 60.0 {
            commands.write_message(UnlockAchievementRequested {
                achievement_id: "speedrunner".into(),
            });
        }
    }
}
```

---

## 📊 Aggregation System

### Aggregation Algorithm

**Implementation**: Pure functions in `registry.rs`

**Percentile Calculation** (P50, P95, P99):
```rust
fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    let index = (sorted_values.len() as f64 * percentile / 100.0).ceil() as usize;
    let clamped_index = index.clamp(1, sorted_values.len()) - 1;
    sorted_values[clamped_index]
}
```

**Rate Calculation**:
```rust
fn calculate_rate(values: &[MetricValue], period_seconds: u64) -> f64 {
    if period_seconds == 0 {
        return 0.0;
    }
    values.len() as f64 / period_seconds as f64
}
```

**Supported Aggregations**:
- **Sum**: Total of all values
- **Count**: Number of values
- **Average**: Mean value
- **Min/Max**: Extremes
- **P50/P95/P99**: Percentiles (requires sorted values)
- **Last**: Most recent value
- **Rate**: Events per second

---

## ✅ Success Criteria

### Functional Requirements

- [ ] **Metric Registration**: Define metrics with types, tags, labels
- [ ] **Value Recording**: Record measurements with timestamps and metadata
- [ ] **Windowed Storage**: Memory-efficient FIFO storage with configurable limits
- [ ] **Statistical Aggregation**: Support all aggregation types (Sum, Avg, P95, etc.)
- [ ] **Snapshot Creation**: Capture point-in-time state of all metrics
- [ ] **Report Generation**: Generate period-based aggregated reports
- [ ] **Observer Extensibility**: Custom reactions via Bevy observers
- [ ] **Periodic Automation**: Auto-snapshots and reports at configured intervals

### Non-Functional Requirements

- [ ] **Zero Allocation**: Aggregation uses stack-allocated data
- [ ] **O(1) Lookup**: HashMap-based metric access
- [ ] **Memory Bounded**: VecDeque with max_values_per_metric limit
- [ ] **No Global Mutable State**: All state in Resources (thread-safe)
- [ ] **Reflection Support**: All types have `#[derive(Reflect)]`
- [ ] **No Async/Await**: Pure sync systems (Bevy ECS requirement)

### Testing Requirements

- [ ] **UT: Define and Record**: Basic metric definition and recording
- [ ] **UT: Windowed Storage**: Values evicted when limit reached
- [ ] **UT: Aggregations**: All aggregation types produce correct results
- [ ] **UT: Percentiles**: P50/P95/P99 calculations correct
- [ ] **UT: Snapshot**: Snapshot captures all current values
- [ ] **UT: Report**: Report contains correct aggregations for period
- [ ] **UT: Observer**: Custom observers receive events correctly
- [ ] **UT: Periodic**: Auto-snapshots/reports trigger at correct intervals
- [ ] **UT: Remove/Clear**: Metrics can be removed individually or all cleared

---

## 🎯 Design Philosophy

### 1. Global Resource-Based Design

**Single Source of Truth**: One global `MetricsRegistry` Resource.

```rust
// ✅ Good (global registry)
fn record_fps(mut commands: Commands, time: Res<Time>) {
    commands.write_message(RecordMetricRequested {
        value: MetricValue {
            metric_id: MetricId("fps".into()),
            value: 1.0 / time.delta_secs(),
            timestamp: current_timestamp(),
            metadata: serde_json::Value::Null,
        },
    });
}

// ❌ Bad (entity-based design, unnecessary complexity)
fn record_fps(mut query: Query<&mut MetricsRegistry>) {
    // Metrics don't need entity-based design
}
```

### 2. Message-Driven Architecture

**All operations via Messages** (request/notification pattern).

```rust
// ✅ Good (message-driven)
commands.write_message(RecordMetricRequested { value });

// ❌ Bad (direct mutation)
registry.record(value);  // NOT how Bevy ECS works
```

### 3. Stateless Aggregation

**Aggregation functions are pure** (no mutation).

```rust
// ✅ Good (pure function)
pub fn aggregate(
    &self,
    metric_id: &MetricId,
    aggregation: AggregationType,
    period_start: u64,
    period_end: u64,
) -> Option<AggregatedMetric> {
    let values = self.values.get(metric_id)?;
    let filtered: Vec<_> = values.iter()
        .filter(|v| v.timestamp >= period_start && v.timestamp <= period_end)
        .collect();

    Some(calculate_aggregation(&filtered, aggregation))
}

// ❌ Bad (stateful aggregation)
pub fn aggregate(&mut self, ...) {
    self.cached_aggregation = ...;  // Don't cache, compute on-demand
}
```

### 4. Memory Efficiency

**Windowed storage prevents unbounded growth**.

```rust
// ✅ Good (bounded storage)
impl MetricsRegistry {
    pub fn record(&mut self, value: MetricValue) -> Result<()> {
        let values = self.values.entry(value.metric_id.clone())
            .or_insert_with(VecDeque::new);

        values.push_back(value);

        // Evict oldest if over limit
        while values.len() > self.config.max_values_per_metric {
            values.pop_front();
        }

        Ok(())
    }
}
```

### 5. Observer-Based Extensibility

**Observers instead of trait hooks** (composition over inheritance).

```rust
// ❌ ISSUN v0.6 approach (trait hook)
#[async_trait]
trait MetricsHook {
    async fn on_metric_recorded(&self, value: &MetricValue);
}

// ✅ Bevy approach (observer pattern)
app.observe(my_metric_observer);

fn my_metric_observer(trigger: Trigger<MetricRecordedEvent>) {
    // Custom reaction...
}
```

---

## 🔮 Future Extensions

### Phase 3+

**Not in Phase 2, but designed for easy addition:**

1. **Metric Metadata Queries**
   - Search metrics by tags
   - Filter metrics by labels
   - List all metrics of a type

2. **Advanced Aggregations**
   - Standard deviation
   - Moving averages
   - Custom aggregation functions

3. **Metric Expressions**
   - Derived metrics (e.g., `cpu_total = cpu_user + cpu_system`)
   - Alerting rules (e.g., `alert if fps < 30 for 10s`)

4. **External Integrations**
   - Prometheus exporter
   - Datadog integration
   - StatsD protocol support

5. **Distributed Metrics**
   - Multi-server aggregation
   - Metrics replication
   - Distributed tracing

6. **Metric Visualization**
   - Real-time graphs
   - Heatmaps
   - Histograms

---

## 📚 Related Plugins

### Dependencies

- **TimePlugin** (optional): Provides `DayChanged` events for periodic snapshots/reports
- No required dependencies (standalone plugin)

### Integration Points

- **PerformancePlugin**: FPS, frame time metrics
- **EconomyPlugin**: Currency flow, transaction metrics
- **CombatPlugin**: Damage dealt, actions per minute
- **AIPlugin**: Decision times, pathfinding performance
- **NetworkPlugin**: Latency, bandwidth, connection count

---

## 🧪 Implementation Strategy

### Phase 2: Core Mechanics (Design)

**Deliverables**:
- [x] Architecture overview complete
- [x] Resource design complete
- [x] Data types defined
- [x] Message definitions complete
- [x] System flow documented
- [x] Observer pattern defined
- [x] Aggregation system documented
- [x] Migration notes written

### Phase 2: Implementation (Next Steps)

**Tasks**:

1. **Create files** (1h):
   ```
   crates/issun-bevy/src/plugins/metrics/
   ├── mod.rs
   ├── types.rs        (MetricId, MetricType, MetricValue, etc.)
   ├── events.rs       (Messages and Events)
   ├── resources.rs    (MetricsRegistry, MetricsConfig, MetricsHistory)
   ├── aggregation.rs  (Aggregation functions)
   ├── systems.rs      (6 processing systems + 2 periodic systems)
   ├── plugin.rs       (Plugin impl)
   └── tests.rs        (UTs)
   ```

2. **Implement Core Types** (2h):
   - MetricId, MetricType, MetricValue
   - MetricDefinition
   - AggregationType, AggregatedMetric
   - MetricSnapshot, MetricReport

3. **Implement MetricsRegistry** (3h):
   - Resource definition
   - define(), record(), remove(), clear()
   - all_values() for snapshots
   - Windowed storage with VecDeque

4. **Implement Aggregation Functions** (3h):
   - Sum, Count, Average, Min, Max
   - Percentiles (P50, P95, P99)
   - Last, Rate
   - Pure functions, no mutation

5. **Implement Systems** (6h):
   - process_define_requests
   - process_record_requests
   - process_snapshot_requests
   - process_report_requests
   - process_remove_requests
   - process_clear_requests
   - periodic_snapshot_system
   - periodic_report_system
   - trigger_observer_events

6. **Implement Plugin** (1h):
   - MetricsPlugin struct
   - Plugin build() method
   - Register all types
   - Add all messages/events
   - Add all systems with ordering

7. **Write Tests** (4h):
   - Define and record test
   - Windowed storage test
   - All aggregation tests (9 types)
   - Snapshot test
   - Report test
   - Observer test
   - Periodic automation test
   - Remove/clear test

8. **Run CHECK STEP** (5min):
   ```bash
   make preflight-bevy
   ```

**Total Estimate**: 20 hours (2-3 days)

### Phase 3+: Extensions

- Advanced aggregations
- External integrations
- Metric visualization
- Distributed metrics

---

## 📋 Migration Notes (ISSUN v0.6 → Bevy)

### Key Changes

| ISSUN v0.6 | Bevy ECS | Reason |
|------------|----------|--------|
| `MetricsRegistry` (runtime state) | `MetricsRegistry` (Resource) | Bevy's resource system |
| `MetricsHook` (trait) | Observer pattern | Bevy's extension mechanism |
| `EventBus` | `MessageWriter` / `MessageReader` | Bevy 0.17 messaging |
| `async fn` | Sync systems | Bevy's synchronous ECS |
| `System` trait | Bevy systems | Native ECS systems |
| `.iter().cloned().collect()` | Message iteration | Bevy's message API |
| `process_events()` method | Separate systems | One system per request type |

### Hook → Observer Migration

| ISSUN v0.6 | Bevy ECS |
|------------|----------|
| `#[async_trait] impl MetricsHook` | `fn observer(trigger: Trigger<...>)` |
| Trait methods (6 hooks) | Observer functions (3 events) |
| `async fn` (async runtime) | Sync systems (Bevy ECS) |
| Called within system | Triggered after state changes |

**Impact**:
- Simpler to extend (just register observers)
- No async complexity
- Better performance (no Arc overhead)
- Reduced to 3 observer events (only user-relevant ones)

### Event Migration

| ISSUN v0.6 | Bevy ECS |
|------------|----------|
| Command Events (6) | Command Messages (6) |
| State Events (6) | State Messages (6) |
| No observer events | Observer Events (3) |

**Impact**:
- Same command/state events structure
- NEW: Observer events for extensibility (not in v0.6)

### Reflect Requirements

**All Bevy types must have**:
- Resources: `#[derive(Reflect)]` + `#[reflect(Resource)]`
- Messages: `#[derive(Reflect)]` + `#[reflect(opaque)]` (NOT `#[reflect(Message)]`)
- Events: No Reflect (observer events don't need it)
- Types: `#[derive(Reflect)]` + `#[reflect(opaque)]`
- Plugin registration: `app.register_type::<T>()`

**Enforcement**: Static linting via `tests/lints.rs` in `make preflight-bevy`

### No Async/Await

**ISSUN v0.6**:
```rust
async fn process_events(&mut self, ...) {
    self.process_define_requests(resources).await;
    self.process_record_requests(resources).await;
    // ...
}

async fn process_define_requests(&mut self, resources: &mut ResourceContext) {
    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
        // ...
    }
}
```

**Bevy ECS**:
```rust
fn process_define_requests(
    mut commands: Commands,
    mut messages: MessageReader<DefineMetricRequested>,
    mut registry: ResMut<MetricsRegistry>,
) {
    for request in messages.read() {
        registry.define(request.definition.clone());
        commands.write_message(MetricDefined {
            definition: request.definition.clone(),
        });
    }
}
```

**Impact**:
- No async/await keywords
- Direct resource access via `Res`/`ResMut`
- Simpler code, better performance
- One system per request type (not one big system)

---

## 🎬 Implementation Checklist

### Type Implementation

- [ ] `MetricId` struct with `#[reflect(opaque)]`
- [ ] `MetricType` enum with `#[reflect(opaque)]`
- [ ] `MetricValue` struct with `#[reflect(opaque)]`
- [ ] `MetricDefinition` struct with `#[reflect(opaque)]`
- [ ] `AggregationType` enum with `#[reflect(opaque)]`
- [ ] `AggregatedMetric` struct with `#[reflect(opaque)]`
- [ ] `MetricSnapshot` struct with `#[reflect(opaque)]`
- [ ] `MetricReport` struct with `#[reflect(opaque)]`

### Resource Implementation

- [ ] `MetricsRegistry` resource with `#[derive(Reflect)]` + `#[reflect(Resource)]`
- [ ] `MetricsConfig` resource with `#[derive(Reflect)]` + `#[reflect(Resource)]`
- [ ] `MetricsHistory` resource with `#[derive(Reflect)]` + `#[reflect(Resource)]`

### Message Implementation

- [ ] `DefineMetricRequested` message
- [ ] `RecordMetricRequested` message
- [ ] `CreateSnapshotRequested` message
- [ ] `GenerateReportRequested` message
- [ ] `RemoveMetricRequested` message
- [ ] `ClearMetricsRequested` message
- [ ] `MetricDefined` message
- [ ] `MetricRecorded` message
- [ ] `SnapshotCreated` message
- [ ] `ReportGenerated` message
- [ ] `MetricRemoved` message
- [ ] `MetricsCleared` message
- [ ] All messages have `#[derive(Reflect)]` + `#[reflect(opaque)]`

### Event Implementation

- [ ] `MetricRecordedEvent` event (no Reflect)
- [ ] `SnapshotCreatedEvent` event (no Reflect)
- [ ] `ReportGeneratedEvent` event (no Reflect)

### System Implementation

- [ ] `process_define_requests` system
- [ ] `process_record_requests` system
- [ ] `process_snapshot_requests` system
- [ ] `process_report_requests` system
- [ ] `process_remove_requests` system
- [ ] `process_clear_requests` system
- [ ] `periodic_snapshot_system` system
- [ ] `periodic_report_system` system
- [ ] `trigger_observer_events` system
- [ ] All systems use `IssunSet` for ordering

### Aggregation Implementation

- [ ] `calculate_sum` function
- [ ] `calculate_count` function
- [ ] `calculate_average` function
- [ ] `calculate_min` function
- [ ] `calculate_max` function
- [ ] `calculate_percentile` function (P50, P95, P99)
- [ ] `calculate_last` function
- [ ] `calculate_rate` function
- [ ] All functions are pure (no mutation)

### Plugin Implementation

- [ ] `MetricsPlugin` struct
- [ ] Plugin `build()` method
- [ ] `app.register_type::<T>()` for all types (18 types)
- [ ] `app.add_message::<M>()` for all messages (12 messages)
- [ ] `app.add_event::<E>()` for observer events (3 events)
- [ ] Systems added with correct ordering

### Testing Implementation

- [ ] Define and record test
- [ ] Windowed storage test (eviction)
- [ ] Sum aggregation test
- [ ] Count aggregation test
- [ ] Average aggregation test
- [ ] Min aggregation test
- [ ] Max aggregation test
- [ ] P50 aggregation test
- [ ] P95 aggregation test
- [ ] P99 aggregation test
- [ ] Last aggregation test
- [ ] Rate aggregation test
- [ ] Snapshot test (all values captured)
- [ ] Report test (correct aggregations)
- [ ] Observer test (events triggered)
- [ ] Periodic snapshot test
- [ ] Periodic report test
- [ ] Remove metric test
- [ ] Clear metrics test
- [ ] `make preflight-bevy` passes

---

## 📖 Usage Summary

### Basic Setup

```rust
use bevy::prelude::*;
use issun_bevy::plugins::metrics::{MetricsPlugin, MetricsConfig};

App::new()
    .add_plugins(MetricsPlugin::default())
    .run();

// Or with custom config
App::new()
    .add_plugins(MetricsPlugin::default().with_config(MetricsConfig {
        max_values_per_metric: 2000,
        enable_periodic_snapshots: true,
        snapshot_period: 1,
        enable_auto_report: true,
        report_period: 7,
    }))
    .run();
```

### Define Metrics

```rust
fn setup_metrics(mut commands: Commands) {
    commands.write_message(DefineMetricRequested {
        definition: MetricDefinition {
            metric_id: MetricId("fps".into()),
            metric_type: MetricType::Gauge,
            description: "Frames per second".into(),
            tags: vec!["performance".into()],
            labels: HashMap::new(),
        },
    });
}
```

### Record Metrics

```rust
fn record_fps(mut commands: Commands, time: Res<Time>) {
    commands.write_message(RecordMetricRequested {
        value: MetricValue {
            metric_id: MetricId("fps".into()),
            value: 1.0 / time.delta_secs(),
            timestamp: current_timestamp(),
            metadata: serde_json::Value::Null,
        },
    });
}
```

### Generate Reports

```rust
fn generate_weekly_report(mut commands: Commands) {
    let now = current_timestamp();
    let week_ago = now - 7 * 24 * 3600;

    commands.write_message(GenerateReportRequested {
        period_start: week_ago,
        period_end: now,
        metric_ids: vec![MetricId("fps".into())],
        aggregations: vec![
            AggregationType::Average,
            AggregationType::P95,
            AggregationType::Min,
        ],
        label: Some("weekly_performance".into()),
    });
}
```

### Custom Observers

```rust
fn setup_observers(app: &mut App) {
    app.observe(fps_alert_observer);
}

fn fps_alert_observer(
    trigger: Trigger<MetricRecordedEvent>,
    mut commands: Commands,
) {
    let value = &trigger.event().value;

    if value.metric_id == MetricId("fps".into()) && value.value < 30.0 {
        warn!("FPS dropped below 30: {}", value.value);
        // Trigger alert
    }
}
```

---

**End of Design Document**
