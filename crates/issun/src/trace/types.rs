//! Types for event chain tracing

use serde::{Deserialize, Serialize};

/// トレースエントリ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceEntry {
    /// フレーム番号
    pub frame: u64,

    /// 発生時刻（ミリ秒）
    pub timestamp_ms: f64,

    /// トレースエントリの種類
    pub entry_type: TraceEntryType,

    /// 発生元（プラグイン名、システム名など）
    pub source: String,

    /// 対象（オプション）
    pub target: Option<String>,
}

impl TraceEntry {
    /// Create a new trace entry
    pub fn new(
        frame: u64,
        timestamp_ms: f64,
        entry_type: TraceEntryType,
        source: impl Into<String>,
    ) -> Self {
        Self {
            frame,
            timestamp_ms,
            entry_type,
            source: source.into(),
            target: None,
        }
    }

    /// Set target
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}

/// トレースエントリの種類
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TraceEntryType {
    /// イベントがpublishされた
    EventPublished {
        event_type: String,
        event_id: String,
    },

    /// イベントがdispatchされた
    EventDispatched {
        event_type: String,
        subscriber_count: usize,
    },

    /// Hookが呼ばれた
    HookCalled {
        hook_name: String,
        plugin: String,
        args: String,
    },

    /// Hookが完了した
    HookCompleted {
        hook_name: String,
        plugin: String,
        duration_ms: f64,
        result: HookResult,
    },
}

/// Hook実行結果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HookResult {
    Success,
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_entry_creation() {
        let entry = TraceEntry::new(
            0,
            0.0,
            TraceEntryType::EventPublished {
                event_type: "TestEvent".to_string(),
                event_id: "test_1".to_string(),
            },
            "EventBus",
        );

        assert_eq!(entry.frame, 0);
        assert_eq!(entry.timestamp_ms, 0.0);
        assert_eq!(entry.source, "EventBus");
        assert!(entry.target.is_none());
    }

    #[test]
    fn test_trace_entry_with_target() {
        let entry = TraceEntry::new(
            1,
            16.0,
            TraceEntryType::HookCalled {
                hook_name: "on_travel_started".to_string(),
                plugin: "WorldMapPlugin".to_string(),
                args: "()".to_string(),
            },
            "WorldMapSystem",
        )
        .with_target("LogisticsPlugin");

        assert_eq!(entry.target, Some("LogisticsPlugin".to_string()));
    }

    #[test]
    fn test_serialization() {
        let entry = TraceEntry::new(
            0,
            0.0,
            TraceEntryType::EventPublished {
                event_type: "TestEvent".to_string(),
                event_id: "test_1".to_string(),
            },
            "EventBus",
        );

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TraceEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.frame, deserialized.frame);
        assert_eq!(entry.source, deserialized.source);
    }
}
