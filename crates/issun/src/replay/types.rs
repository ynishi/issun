//! Types for event recording and replay

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 記録されたイベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// フレーム番号
    pub frame: u64,

    /// 発生時刻（ミリ秒）
    pub timestamp_ms: f64,

    /// イベント型名
    pub event_type: String,

    /// シリアライズされたイベント本体
    pub payload: Vec<u8>,
}

impl RecordedEvent {
    /// Create a new recorded event
    pub fn new(frame: u64, timestamp_ms: f64, event_type: String, payload: Vec<u8>) -> Self {
        Self {
            frame,
            timestamp_ms,
            event_type,
            payload,
        }
    }
}

/// レコーディング統計情報
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingStats {
    /// 総イベント数
    pub total_events: usize,

    /// 総フレーム数
    pub total_frames: u64,

    /// 総時間（ミリ秒）
    pub duration_ms: f64,

    /// イベント型別カウント
    pub event_types: HashMap<String, usize>,
}

impl RecordingStats {
    /// Create empty stats
    pub fn new() -> Self {
        Self {
            total_events: 0,
            total_frames: 0,
            duration_ms: 0.0,
            event_types: HashMap::new(),
        }
    }

    /// Add an event to stats
    pub fn add_event(&mut self, event_type: &str, frame: u64, timestamp_ms: f64) {
        self.total_events += 1;
        self.total_frames = self.total_frames.max(frame + 1);
        self.duration_ms = self.duration_ms.max(timestamp_ms);
        *self.event_types.entry(event_type.to_string()).or_insert(0) += 1;
    }
}

impl Default for RecordingStats {
    fn default() -> Self {
        Self::new()
    }
}

/// レコーディングメタデータ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// 作成日時
    pub created_at: String,

    /// バージョン
    pub version: u32,

    /// 統計情報
    pub stats: RecordingStats,
}

impl RecordingMetadata {
    /// Create new metadata
    pub fn new(stats: RecordingStats) -> Self {
        Self {
            created_at: chrono::Utc::now().to_rfc3339(),
            version: 1,
            stats,
        }
    }
}

/// レコーディングファイル形式
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingFile {
    /// メタデータ
    pub metadata: RecordingMetadata,

    /// 記録されたイベント
    pub recordings: Vec<RecordedEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorded_event_creation() {
        let event = RecordedEvent::new(0, 0.0, "TestEvent".to_string(), vec![1, 2, 3]);

        assert_eq!(event.frame, 0);
        assert_eq!(event.timestamp_ms, 0.0);
        assert_eq!(event.event_type, "TestEvent");
        assert_eq!(event.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_recording_stats() {
        let mut stats = RecordingStats::new();

        stats.add_event("Event1", 0, 0.0);
        stats.add_event("Event1", 1, 16.0);
        stats.add_event("Event2", 2, 32.0);

        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.total_frames, 3);
        assert_eq!(stats.duration_ms, 32.0);
        assert_eq!(*stats.event_types.get("Event1").unwrap(), 2);
        assert_eq!(*stats.event_types.get("Event2").unwrap(), 1);
    }

    #[test]
    fn test_serialization() {
        let event = RecordedEvent::new(0, 0.0, "TestEvent".to_string(), vec![1, 2, 3]);

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RecordedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.frame, deserialized.frame);
        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.payload, deserialized.payload);
    }

    #[test]
    fn test_recording_file() {
        let mut stats = RecordingStats::new();
        stats.add_event("Event1", 0, 0.0);

        let metadata = RecordingMetadata::new(stats);
        let file = RecordingFile {
            metadata,
            recordings: vec![RecordedEvent::new(
                0,
                0.0,
                "Event1".to_string(),
                vec![1, 2, 3],
            )],
        };

        let json = serde_json::to_string(&file).unwrap();
        let deserialized: RecordingFile = serde_json::from_str(&json).unwrap();

        assert_eq!(file.recordings.len(), deserialized.recordings.len());
    }
}
