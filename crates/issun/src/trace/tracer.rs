//! Event chain tracer implementation

use super::types::{TraceEntry, TraceEntryType};
use std::collections::HashMap;
use std::time::Instant;

/// Event/Hook呼び出しトレーサー
pub struct EventChainTracer {
    traces: Vec<TraceEntry>,
    enabled: bool,
    start_time: Instant,
    current_frame: u64,
}

impl EventChainTracer {
    /// 新しいトレーサーを作成
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            enabled: false,
            start_time: Instant::now(),
            current_frame: 0,
        }
    }

    /// トレースを有効化
    pub fn enable(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
    }

    /// トレースを無効化
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// トレースが有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 現在のフレームを設定
    pub fn set_frame(&mut self, frame: u64) {
        self.current_frame = frame;
    }

    /// 現在のフレームを取得
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// 現在の経過時間（ミリ秒）を取得
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64() * 1000.0
    }

    /// トレースエントリを記録
    pub fn record(&mut self, entry: TraceEntry) {
        if !self.enabled {
            return;
        }
        self.traces.push(entry);
    }

    /// トレースエントリを記録（簡易版）
    pub fn record_simple(&mut self, entry_type: TraceEntryType, source: impl Into<String>) {
        if !self.enabled {
            return;
        }

        self.traces.push(TraceEntry {
            frame: self.current_frame,
            timestamp_ms: self.elapsed_ms(),
            entry_type,
            source: source.into(),
            target: None,
        });
    }

    /// トレースをクリア
    pub fn clear(&mut self) {
        self.traces.clear();
        self.start_time = Instant::now();
        self.current_frame = 0;
    }

    /// 全トレースエントリを取得
    pub fn traces(&self) -> &[TraceEntry] {
        &self.traces
    }

    /// 特定フレームのトレースを取得
    pub fn traces_for_frame(&self, frame: u64) -> Vec<&TraceEntry> {
        self.traces.iter().filter(|e| e.frame == frame).collect()
    }

    /// 特定フレーム範囲のトレースを取得
    pub fn traces_for_range(&self, start: u64, end: u64) -> Vec<&TraceEntry> {
        self.traces
            .iter()
            .filter(|e| e.frame >= start && e.frame <= end)
            .collect()
    }

    /// 統計情報を取得
    pub fn stats(&self) -> TracerStats {
        let mut event_types: HashMap<String, usize> = HashMap::new();
        let mut hook_calls: HashMap<String, usize> = HashMap::new();

        for entry in &self.traces {
            match &entry.entry_type {
                TraceEntryType::EventPublished { event_type, .. } => {
                    *event_types.entry(event_type.clone()).or_insert(0) += 1;
                }
                TraceEntryType::HookCalled { hook_name, .. } => {
                    *hook_calls.entry(hook_name.clone()).or_insert(0) += 1;
                }
                _ => {}
            }
        }

        TracerStats {
            total_entries: self.traces.len(),
            total_frames: self.current_frame + 1,
            event_types,
            hook_calls,
        }
    }

    /// JSONエクスポート
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.traces)
    }

    /// ファイルに保存
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.export_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// ファイルから読み込み
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let traces: Vec<TraceEntry> = serde_json::from_str(&json)?;

        Ok(Self {
            traces,
            enabled: false,
            start_time: Instant::now(),
            current_frame: 0,
        })
    }
}

impl Default for EventChainTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// トレーサー統計情報
#[derive(Clone, Debug)]
pub struct TracerStats {
    pub total_entries: usize,
    pub total_frames: u64,
    pub event_types: HashMap<String, usize>,
    pub hook_calls: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::types::TraceEntryType;

    #[test]
    fn test_tracer_creation() {
        let tracer = EventChainTracer::new();
        assert!(!tracer.is_enabled());
        assert_eq!(tracer.traces().len(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut tracer = EventChainTracer::new();

        tracer.enable();
        assert!(tracer.is_enabled());

        tracer.disable();
        assert!(!tracer.is_enabled());
    }

    #[test]
    fn test_record_when_disabled() {
        let mut tracer = EventChainTracer::new();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "TestEvent".to_string(),
                event_id: "test_1".to_string(),
            },
            "EventBus",
        );

        assert_eq!(tracer.traces().len(), 0);
    }

    #[test]
    fn test_record_when_enabled() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "TestEvent".to_string(),
                event_id: "test_1".to_string(),
            },
            "EventBus",
        );

        assert_eq!(tracer.traces().len(), 1);
        assert_eq!(tracer.traces()[0].source, "EventBus");
    }

    #[test]
    fn test_frame_tracking() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.set_frame(0);
        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "Event1".to_string(),
                event_id: "1".to_string(),
            },
            "EventBus",
        );

        tracer.set_frame(1);
        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "Event2".to_string(),
                event_id: "2".to_string(),
            },
            "EventBus",
        );

        let frame_0 = tracer.traces_for_frame(0);
        let frame_1 = tracer.traces_for_frame(1);

        assert_eq!(frame_0.len(), 1);
        assert_eq!(frame_1.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "TestEvent".to_string(),
                event_id: "test_1".to_string(),
            },
            "EventBus",
        );

        assert_eq!(tracer.traces().len(), 1);

        tracer.clear();
        assert_eq!(tracer.traces().len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "Event1".to_string(),
                event_id: "1".to_string(),
            },
            "EventBus",
        );

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "Event1".to_string(),
                event_id: "2".to_string(),
            },
            "EventBus",
        );

        tracer.record_simple(
            TraceEntryType::HookCalled {
                hook_name: "on_test".to_string(),
                plugin: "TestPlugin".to_string(),
                args: "()".to_string(),
            },
            "TestSystem",
        );

        let stats = tracer.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(*stats.event_types.get("Event1").unwrap(), 2);
        assert_eq!(*stats.hook_calls.get("on_test").unwrap(), 1);
    }

    #[test]
    fn test_save_and_load() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "TestEvent".to_string(),
                event_id: "test_1".to_string(),
            },
            "EventBus",
        );

        let temp_file = "/tmp/test_tracer.json";
        tracer.save(temp_file).unwrap();

        let loaded = EventChainTracer::load(temp_file).unwrap();
        assert_eq!(loaded.traces().len(), 1);
        assert_eq!(loaded.traces()[0].source, "EventBus");

        std::fs::remove_file(temp_file).ok();
    }
}
