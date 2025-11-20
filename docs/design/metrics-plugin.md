# MetricsPlugin Design Document

**Status**: Draft
**Created**: 2025-11-21
**Author**: issun team

## 🎯 Overview

MetricsPlugin provides generic metrics collection, aggregation, and reporting for game analytics, KPI tracking, and performance monitoring.

**Use Cases**:
- **Game Analytics**: Track player actions, progression, economy metrics
- **KPI Monitoring**: Revenue, engagement, retention metrics
- **Balance Tuning**: Weapon usage rates, win rates, difficulty metrics
- **A/B Testing**: Compare metrics across test groups
- **Performance**: FPS, latency, memory usage tracking
- **Debug/QA**: Error counts, crash rates, bug reproduction data

## 🏗️ Architecture

### Core Concepts

1. **Metric**: A named measurement (e.g., "player_deaths", "gold_earned")
2. **MetricType**: Counter (累積), Gauge (現在値), Histogram (分布)
3. **MetricValue**: Timestamped value with metadata
4. **Aggregation**: Sum, Count, Average, Min, Max, Percentile
5. **Window**: Time window for aggregation (last N values, last N seconds)
6. **Snapshot**: Point-in-time metric values
7. **Report**: Aggregated metrics over a period

### Key Design Principles

✅ **Generic & Extensible**: No hard-coded metrics (player_deaths, gold_earned, etc.)
✅ **Hook-based Customization**: Game-specific logic via hooks
✅ **Event-driven**: Record metrics via events
✅ **Efficient Aggregation**: Pre-computed statistics, sliding windows
✅ **Easy Reporting**: Built-in report generation (JSON, summary)
✅ **Periodic Reports**: Daily/Weekly/Custom period reports

---

## 📦 Component Structure

```
crates/issun/src/plugin/metrics/
├── mod.rs            # Public exports
├── types.rs          # MetricId, MetricType, MetricValue
├── registry.rs       # MetricsRegistry (Resource)
├── aggregation.rs    # Aggregation logic (Sum, Avg, Percentile)
├── reporting.rs      # MetricReport, MetricSnapshot
├── hook.rs           # MetricsHook trait + DefaultMetricsHook
├── plugin.rs         # MetricsPlugin implementation
├── system.rs         # MetricsSystem (event processing)
└── events.rs         # Command & State events
```

---

## 🧩 Core Types

### `MetricId`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricId(String);

impl MetricId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### `MetricType`

```rust
/// Type of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter: Monotonically increasing value (e.g., total_kills, gold_earned)
    ///
    /// Aggregations: Sum, Count, Rate
    Counter,

    /// Gauge: Point-in-time value that can go up or down (e.g., player_count, hp)
    ///
    /// Aggregations: Average, Min, Max, Last
    Gauge,

    /// Histogram: Distribution of values (e.g., damage_dealt, level_completion_time)
    ///
    /// Aggregations: Average, Min, Max, Percentile, Count
    Histogram,
}
```

### `MetricValue`

```rust
/// A single metric measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Metric identifier
    pub metric_id: MetricId,

    /// Measured value
    pub value: f64,

    /// Timestamp (game time: day, turn, or real time: seconds since epoch)
    pub timestamp: u64,

    /// Optional metadata (e.g., player_id, weapon_type, level_id)
    #[serde(default)]
    pub metadata: serde_json::Value,
}
```

### `MetricDefinition`

```rust
/// Definition of a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: MetricId,
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub unit: String, // e.g., "count", "seconds", "gold", "hp"
}
```

### `AggregationType`

```rust
/// Type of aggregation to perform on metric values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum of all values
    Sum,

    /// Count of values
    Count,

    /// Average (mean) of values
    Average,

    /// Minimum value
    Min,

    /// Maximum value
    Max,

    /// 50th percentile (median)
    P50,

    /// 95th percentile
    P95,

    /// 99th percentile
    P99,

    /// Most recent value
    Last,

    /// Rate (value per unit time)
    Rate,
}
```

### `AggregatedMetric`

```rust
/// Aggregated metric result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub metric_id: MetricId,
    pub aggregation: AggregationType,
    pub value: f64,
    pub count: usize, // Number of samples
    pub period_start: u64,
    pub period_end: u64,
}
```

### `MetricsRegistry`

```rust
/// Registry of all metrics in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsRegistry {
    /// Metric definitions
    definitions: HashMap<MetricId, MetricDefinition>,

    /// Recent values for each metric (windowed)
    values: HashMap<MetricId, VecDeque<MetricValue>>,

    /// Configuration
    config: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Maximum number of values to keep per metric (for windowed aggregation)
    pub max_values_per_metric: usize,

    /// Enable automatic periodic snapshots
    pub enable_periodic_snapshots: bool,

    /// Snapshot period (in game time units, e.g., days)
    pub snapshot_period: u64,

    /// Enable automatic reporting
    pub enable_auto_report: bool,

    /// Report period (in game time units)
    pub report_period: u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_values_per_metric: 1000,
            enable_periodic_snapshots: false,
            snapshot_period: 1, // Daily
            enable_auto_report: false,
            report_period: 7, // Weekly
        }
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            values: HashMap::new(),
            config: MetricsConfig::default(),
        }
    }

    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            definitions: HashMap::new(),
            values: HashMap::new(),
            config,
        }
    }

    /// Define a new metric
    pub fn define(&mut self, definition: MetricDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Record a metric value
    pub fn record(&mut self, value: MetricValue) {
        let values = self.values.entry(value.metric_id.clone()).or_insert_with(VecDeque::new);
        values.push_back(value);

        // Enforce window size
        while values.len() > self.config.max_values_per_metric {
            values.pop_front();
        }
    }

    /// Get aggregated metric for a specific period
    pub fn aggregate(
        &self,
        metric_id: &MetricId,
        aggregation: AggregationType,
        period_start: u64,
        period_end: u64,
    ) -> Option<AggregatedMetric> {
        let values = self.values.get(metric_id)?;

        // Filter values in period
        let filtered: Vec<&MetricValue> = values
            .iter()
            .filter(|v| v.timestamp >= period_start && v.timestamp <= period_end)
            .collect();

        if filtered.is_empty() {
            return None;
        }

        let result = match aggregation {
            AggregationType::Sum => filtered.iter().map(|v| v.value).sum(),
            AggregationType::Count => filtered.len() as f64,
            AggregationType::Average => {
                filtered.iter().map(|v| v.value).sum::<f64>() / filtered.len() as f64
            }
            AggregationType::Min => filtered.iter().map(|v| v.value).fold(f64::INFINITY, f64::min),
            AggregationType::Max => {
                filtered.iter().map(|v| v.value).fold(f64::NEG_INFINITY, f64::max)
            }
            AggregationType::Last => filtered.last().map(|v| v.value).unwrap_or(0.0),
            AggregationType::P50 => calculate_percentile(&filtered, 0.5),
            AggregationType::P95 => calculate_percentile(&filtered, 0.95),
            AggregationType::P99 => calculate_percentile(&filtered, 0.99),
            AggregationType::Rate => {
                let duration = period_end.saturating_sub(period_start).max(1);
                filtered.iter().map(|v| v.value).sum::<f64>() / duration as f64
            }
        };

        Some(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation,
            value: result,
            count: filtered.len(),
            period_start,
            period_end,
        })
    }

    /// Get all values for a metric (within window)
    pub fn get_values(&self, metric_id: &MetricId) -> Option<&VecDeque<MetricValue>> {
        self.values.get(metric_id)
    }

    /// Get metric definition
    pub fn get_definition(&self, metric_id: &MetricId) -> Option<&MetricDefinition> {
        self.definitions.get(metric_id)
    }

    /// List all defined metrics
    pub fn list_metrics(&self) -> Vec<&MetricDefinition> {
        self.definitions.values().collect()
    }
}

/// Helper function to calculate percentile
fn calculate_percentile(values: &[&MetricValue], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted: Vec<f64> = values.iter().map(|v| v.value).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = ((sorted.len() as f64 - 1.0) * percentile) as usize;
    sorted[index]
}
```

---

## 📊 Reporting

### `MetricSnapshot`

```rust
/// Point-in-time snapshot of all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub timestamp: u64,
    pub metrics: Vec<MetricValue>,
}

impl MetricSnapshot {
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            metrics: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, value: MetricValue) {
        self.metrics.push(value);
    }
}
```

### `MetricReport`

```rust
/// Aggregated report for a period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReport {
    pub period_start: u64,
    pub period_end: u64,
    pub metrics: Vec<AggregatedMetric>,
}

impl MetricReport {
    pub fn new(period_start: u64, period_end: u64) -> Self {
        Self {
            period_start,
            period_end,
            metrics: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, metric: AggregatedMetric) {
        self.metrics.push(metric);
    }

    /// Generate a summary report (for logging/UI)
    pub fn to_summary(&self) -> String {
        let mut summary = format!(
            "Metric Report (Period: {} - {})\n",
            self.period_start, self.period_end
        );
        summary.push_str("=================================\n");

        for metric in &self.metrics {
            summary.push_str(&format!(
                "{:?} {}: {:.2} (n={})\n",
                metric.aggregation, metric.metric_id.as_str(), metric.value, metric.count
            ));
        }

        summary
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
```

---

## 🪝 Hook System

```rust
#[async_trait]
pub trait MetricsHook: Send + Sync {
    /// Called when a metric is recorded
    ///
    /// Can be used to trigger alerts, logging, or side effects.
    async fn on_metric_recorded(
        &self,
        _value: &MetricValue,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when a periodic snapshot is generated
    async fn on_snapshot_generated(
        &self,
        _snapshot: &MetricSnapshot,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when a periodic report is generated
    async fn on_report_generated(
        &self,
        _report: &MetricReport,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Validate metric value before recording
    ///
    /// Return `Ok(())` to allow, `Err(reason)` to reject.
    async fn validate_metric(
        &self,
        _value: &MetricValue,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultMetricsHook;

#[async_trait]
impl MetricsHook for DefaultMetricsHook {}
```

---

## 📡 Event System

### Command Events (Request)

```rust
/// Request to record a metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecordRequested {
    pub metric_id: MetricId,
    pub value: f64,
    pub timestamp: u64,
    pub metadata: Option<serde_json::Value>,
}

/// Request to generate a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshotRequested {
    pub timestamp: u64,
}

/// Request to generate a report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReportRequested {
    pub period_start: u64,
    pub period_end: u64,
    pub metric_ids: Vec<MetricId>, // Empty = all metrics
    pub aggregations: Vec<AggregationType>,
}
```

### State Events (Notification)

```rust
/// Published when a metric is recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecordedEvent {
    pub metric_id: MetricId,
    pub value: f64,
    pub timestamp: u64,
}

/// Published when a snapshot is generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshotGeneratedEvent {
    pub snapshot: MetricSnapshot,
}

/// Published when a report is generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReportGeneratedEvent {
    pub report: MetricReport,
}
```

---

## 📝 Usage Examples

### Basic Setup

```rust
use issun::plugin::metrics::{MetricsPlugin, MetricDefinition, MetricType, MetricId};

let game = GameBuilder::new()
    .add_plugin(MetricsPlugin::new())
    .build()
    .await?;

// Define metrics
let mut registry = resources.get_mut::<MetricsRegistry>().await.unwrap();
registry.define(MetricDefinition {
    id: MetricId::new("player_deaths"),
    name: "Player Deaths".into(),
    description: "Total player deaths".into(),
    metric_type: MetricType::Counter,
    unit: "count".into(),
});
```

### Recording Metrics

```rust
// Via event
let mut bus = resources.get_mut::<EventBus>().await.unwrap();
bus.publish(MetricRecordRequested {
    metric_id: MetricId::new("player_deaths"),
    value: 1.0,
    timestamp: game_timer.day,
    metadata: Some(json!({"level": "boss_fight"})),
});

// Or directly
let mut registry = resources.get_mut::<MetricsRegistry>().await.unwrap();
registry.record(MetricValue {
    metric_id: MetricId::new("player_deaths"),
    value: 1.0,
    timestamp: game_timer.day,
    metadata: json!({"level": "boss_fight"}),
});
```

### Generating Reports

```rust
// Request a report
bus.publish(MetricReportRequested {
    period_start: 0,
    period_end: 7, // Last 7 days
    metric_ids: vec![],
    aggregations: vec![
        AggregationType::Sum,
        AggregationType::Average,
        AggregationType::P95,
    ],
});

// Or generate manually
let registry = resources.get::<MetricsRegistry>().await.unwrap();
let mut report = MetricReport::new(0, 7);

for metric_def in registry.list_metrics() {
    if let Some(agg) = registry.aggregate(&metric_def.id, AggregationType::Sum, 0, 7) {
        report.add_metric(agg);
    }
}

println!("{}", report.to_summary());
```

### Custom Hook for Logging

```rust
struct GameMetricsHook;

#[async_trait]
impl MetricsHook for GameMetricsHook {
    async fn on_report_generated(
        &self,
        report: &MetricReport,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!("週次レポート生成: {} metrics", report.metrics.len()));
        }

        // Log to console
        println!("{}", report.to_summary());
    }
}
```

---

## 🎮 Game-Specific Examples

### KPI Dashboard

```rust
// Track key metrics
registry.define(MetricDefinition {
    id: MetricId::new("daily_active_users"),
    name: "Daily Active Users".into(),
    description: "Number of unique players per day".into(),
    metric_type: MetricType::Gauge,
    unit: "users".into(),
});

registry.define(MetricDefinition {
    id: MetricId::new("revenue"),
    name: "Revenue".into(),
    description: "Total revenue earned".into(),
    metric_type: MetricType::Counter,
    unit: "gold".into(),
});

// Generate weekly report
let report = generate_weekly_report(&registry, week_start, week_end);
```

### Balance Tuning

```rust
// Track weapon usage
registry.define(MetricDefinition {
    id: MetricId::new("weapon_plasma_rifle_usage"),
    name: "Plasma Rifle Usage".into(),
    description: "Number of times plasma rifle was used".into(),
    metric_type: MetricType::Counter,
    unit: "count".into(),
});

// Track damage dealt
registry.define(MetricDefinition {
    id: MetricId::new("damage_dealt"),
    name: "Damage Dealt".into(),
    description: "Distribution of damage dealt per attack".into(),
    metric_type: MetricType::Histogram,
    unit: "hp".into(),
});

// Analyze balance
let avg_damage = registry.aggregate(
    &MetricId::new("damage_dealt"),
    AggregationType::Average,
    0, 100
);
let p95_damage = registry.aggregate(
    &MetricId::new("damage_dealt"),
    AggregationType::P95,
    0, 100
);
```

---

## ✅ Design Checklist

- [x] No hard-coded metrics
- [x] Generic & extensible (Counter, Gauge, Histogram)
- [x] Multiple aggregation types (Sum, Count, Avg, Min, Max, Percentile)
- [x] Windowed aggregation (last N values)
- [x] Snapshot support
- [x] Report generation (summary, JSON)
- [x] Periodic reports (optional)
- [x] Hook system for customization
- [x] Event-driven architecture
- [ ] Comprehensive tests (to be written)

---

## 🎓 Key Design Decisions

### 1. Three Metric Types

**Counter**: Monotonically increasing (total_kills)
**Gauge**: Point-in-time value (current_hp)
**Histogram**: Value distribution (damage_dealt)

This covers most game metrics use cases.

### 2. Percentile Calculation

P50/P95/P99 are critical for understanding distributions (latency, damage, time-to-complete).

Simple percentile calculation is good enough for game analytics.

### 3. Windowed Aggregation

Keep last N values per metric to prevent memory bloat.

Default: 1000 values per metric.

### 4. Flexible Timestamp

Use u64 for timestamp (game days, turns, or unix seconds).

Games can choose their time unit.

### 5. Metadata Support

Each metric value can have custom metadata (player_id, level_id, weapon_type).

Enables filtering and grouping in post-processing.

---

**End of Design Document**
