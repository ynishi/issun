//! Prometheus metrics for relay server monitoring

use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Encoder, Gauge,
    HistogramVec, TextEncoder,
};
use std::sync::Arc;

/// Metrics collector for the relay server
#[derive(Clone)]
pub struct Metrics {
    /// Total number of connected clients
    pub connected_clients: Gauge,

    /// Total number of active rooms
    pub active_rooms: Gauge,

    /// Total events relayed (by scope type)
    pub events_relayed: CounterVec,

    /// Client connection duration histogram
    pub connection_duration: HistogramVec,

    /// Event relay latency histogram (microseconds)
    pub relay_latency: HistogramVec,

    /// Total bytes sent
    pub bytes_sent: CounterVec,

    /// Total bytes received
    pub bytes_received: CounterVec,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self, prometheus::Error> {
        Ok(Self {
            connected_clients: register_gauge!(
                "issun_connected_clients",
                "Number of currently connected clients"
            )?,

            active_rooms: register_gauge!("issun_active_rooms", "Number of active game rooms")?,

            events_relayed: register_counter_vec!(
                "issun_events_relayed_total",
                "Total number of events relayed",
                &["scope"] // broadcast, targeted, to_server
            )?,

            connection_duration: register_histogram_vec!(
                "issun_connection_duration_seconds",
                "Duration of client connections in seconds",
                &["status"], // completed, timeout, error
                vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]
            )?,

            relay_latency: register_histogram_vec!(
                "issun_relay_latency_microseconds",
                "Time to relay an event in microseconds",
                &["scope"],
                vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0]
            )?,

            bytes_sent: register_counter_vec!(
                "issun_bytes_sent_total",
                "Total bytes sent to clients",
                &["client_id"]
            )?,

            bytes_received: register_counter_vec!(
                "issun_bytes_received_total",
                "Total bytes received from clients",
                &["client_id"]
            )?,
        })
    }

    /// Record a relayed event
    pub fn record_event_relayed(&self, scope: &str) {
        self.events_relayed.with_label_values(&[scope]).inc();
    }

    /// Record relay latency
    pub fn record_relay_latency(&self, scope: &str, duration_micros: f64) {
        self.relay_latency
            .with_label_values(&[scope])
            .observe(duration_micros);
    }

    /// Record connection duration
    pub fn record_connection_duration(&self, status: &str, duration_secs: f64) {
        self.connection_duration
            .with_label_values(&[status])
            .observe(duration_secs);
    }

    /// Increment connected clients
    pub fn increment_connected_clients(&self) {
        self.connected_clients.inc();
    }

    /// Decrement connected clients
    pub fn decrement_connected_clients(&self) {
        self.connected_clients.dec();
    }

    /// Set active rooms count
    pub fn set_active_rooms(&self, count: usize) {
        self.active_rooms.set(count as f64);
    }

    /// Record bytes sent
    pub fn record_bytes_sent(&self, client_id: &str, bytes: usize) {
        self.bytes_sent
            .with_label_values(&[client_id])
            .inc_by(bytes as f64);
    }

    /// Record bytes received
    pub fn record_bytes_received(&self, client_id: &str, bytes: usize) {
        self.bytes_received
            .with_label_values(&[client_id])
            .inc_by(bytes as f64);
    }

    /// Gather and encode all metrics as Prometheus text format
    pub fn gather(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| {
            prometheus::Error::Msg(format!("Failed to encode metrics to UTF-8: {}", e))
        })
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}

/// Shared metrics instance wrapped in Arc for thread-safe access
pub type SharedMetrics = Arc<Metrics>;
