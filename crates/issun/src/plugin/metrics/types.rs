//! Metrics types and data structures

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a metric
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricId(String);

impl MetricId {
    /// Create a new metric identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MetricId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MetricId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MetricId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

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

impl MetricValue {
    /// Create a new metric value
    pub fn new(metric_id: MetricId, value: f64, timestamp: u64) -> Self {
        Self {
            metric_id,
            value,
            timestamp,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a metric value with metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Definition of a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: MetricId,
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub unit: String, // e.g., "count", "seconds", "gold", "hp"
}

impl MetricDefinition {
    /// Create a new metric definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        metric_type: MetricType,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            id: MetricId::new(id),
            name: name.into(),
            description: description.into(),
            metric_type,
            unit: unit.into(),
        }
    }
}

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

impl AggregatedMetric {
    /// Create a new aggregated metric
    pub fn new(
        metric_id: MetricId,
        aggregation: AggregationType,
        value: f64,
        count: usize,
        period_start: u64,
        period_end: u64,
    ) -> Self {
        Self {
            metric_id,
            aggregation,
            value,
            count,
            period_start,
            period_end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_id_creation() {
        let id = MetricId::new("test_metric");
        assert_eq!(id.as_str(), "test_metric");
        assert_eq!(id.to_string(), "test_metric");
    }

    #[test]
    fn test_metric_id_from_string() {
        let id: MetricId = "test_metric".into();
        assert_eq!(id.as_str(), "test_metric");
    }

    #[test]
    fn test_metric_value_creation() {
        let value = MetricValue::new(MetricId::new("test"), 42.0, 100);
        assert_eq!(value.metric_id.as_str(), "test");
        assert_eq!(value.value, 42.0);
        assert_eq!(value.timestamp, 100);
    }

    #[test]
    fn test_metric_value_with_metadata() {
        let value = MetricValue::new(MetricId::new("test"), 42.0, 100)
            .with_metadata(serde_json::json!({"level": "boss_fight"}));
        assert_eq!(value.metadata["level"], "boss_fight");
    }

    #[test]
    fn test_metric_definition_creation() {
        let def = MetricDefinition::new(
            "player_deaths",
            "Player Deaths",
            "Total player deaths",
            MetricType::Counter,
            "count",
        );
        assert_eq!(def.id.as_str(), "player_deaths");
        assert_eq!(def.name, "Player Deaths");
        assert_eq!(def.metric_type, MetricType::Counter);
        assert_eq!(def.unit, "count");
    }

    #[test]
    fn test_aggregated_metric_creation() {
        let agg = AggregatedMetric::new(
            MetricId::new("test"),
            AggregationType::Sum,
            100.0,
            10,
            0,
            100,
        );
        assert_eq!(agg.metric_id.as_str(), "test");
        assert_eq!(agg.aggregation, AggregationType::Sum);
        assert_eq!(agg.value, 100.0);
        assert_eq!(agg.count, 10);
    }
}
