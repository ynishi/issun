//! Event recorder implementation

use super::types::{RecordedEvent, RecordingFile, RecordingMetadata, RecordingStats};
use serde::Serialize;
use std::time::Instant;

/// イベントレコーダー
pub struct EventRecorder {
    recordings: Vec<RecordedEvent>,
    start_time: Instant,
    enabled: bool,
    current_frame: u64,
}

impl EventRecorder {
    /// 新しいレコーダーを作成
    pub fn new() -> Self {
        Self {
            recordings: Vec::new(),
            start_time: Instant::now(),
            enabled: false,
            current_frame: 0,
        }
    }

    /// 記録を開始
    pub fn start(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
    }

    /// 記録を停止
    pub fn stop(&mut self) {
        self.enabled = false;
    }

    /// 記録中かどうか
    pub fn is_recording(&self) -> bool {
        self.enabled
    }

    /// フレームを設定
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

    /// イベントを記録
    pub fn record<E: crate::event::Event + Serialize>(&mut self, event: &E) {
        if !self.enabled {
            return;
        }

        let payload = match bincode::serialize(event) {
            Ok(p) => p,
            Err(_) => return, // Serialization failed, skip
        };

        let event_type = std::any::type_name::<E>().to_string();
        let timestamp_ms = self.elapsed_ms();

        self.recordings.push(RecordedEvent::new(
            self.current_frame,
            timestamp_ms,
            event_type,
            payload,
        ));
    }

    /// 記録をクリア
    pub fn clear(&mut self) {
        self.recordings.clear();
        self.start_time = Instant::now();
        self.current_frame = 0;
    }

    /// 全記録を取得
    pub fn recordings(&self) -> &[RecordedEvent] {
        &self.recordings
    }

    /// 統計情報を取得
    pub fn stats(&self) -> RecordingStats {
        let mut stats = RecordingStats::new();

        for recording in &self.recordings {
            stats.add_event(&recording.event_type, recording.frame, recording.timestamp_ms);
        }

        stats
    }

    /// ファイルに保存
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = RecordingMetadata::new(self.stats());
        let file = RecordingFile {
            metadata,
            recordings: self.recordings.clone(),
        };

        let file_handle = std::fs::File::create(path)?;
        bincode::serialize_into(file_handle, &file)?;
        Ok(())
    }

    /// ファイルから読み込み
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_handle = std::fs::File::open(path)?;
        let file: RecordingFile = bincode::deserialize_from(file_handle)?;

        Ok(Self {
            recordings: file.recordings,
            start_time: Instant::now(),
            enabled: false,
            current_frame: 0,
        })
    }
}

impl Default for EventRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct TestEvent {
        value: i32,
    }

    impl crate::event::Event for TestEvent {}

    #[test]
    fn test_recorder_creation() {
        let recorder = EventRecorder::new();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.recordings().len(), 0);
    }

    #[test]
    fn test_start_stop() {
        let mut recorder = EventRecorder::new();

        recorder.start();
        assert!(recorder.is_recording());

        recorder.stop();
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_record_when_disabled() {
        let mut recorder = EventRecorder::new();

        recorder.record(&TestEvent { value: 42 });

        assert_eq!(recorder.recordings().len(), 0);
    }

    #[test]
    fn test_record_when_enabled() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.record(&TestEvent { value: 42 });

        assert_eq!(recorder.recordings().len(), 1);
        assert!(recorder.recordings()[0].event_type.contains("TestEvent"));
    }

    #[test]
    fn test_frame_tracking() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.set_frame(0);
        recorder.record(&TestEvent { value: 1 });

        recorder.set_frame(1);
        recorder.record(&TestEvent { value: 2 });

        recorder.set_frame(2);
        recorder.record(&TestEvent { value: 3 });

        let recordings = recorder.recordings();
        assert_eq!(recordings.len(), 3);
        assert_eq!(recordings[0].frame, 0);
        assert_eq!(recordings[1].frame, 1);
        assert_eq!(recordings[2].frame, 2);
    }

    #[test]
    fn test_clear() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.record(&TestEvent { value: 42 });
        assert_eq!(recorder.recordings().len(), 1);

        recorder.clear();
        assert_eq!(recorder.recordings().len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.set_frame(0);
        recorder.record(&TestEvent { value: 1 });

        recorder.set_frame(1);
        recorder.record(&TestEvent { value: 2 });

        let stats = recorder.stats();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_frames, 2);
    }

    #[test]
    fn test_save_and_load() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.record(&TestEvent { value: 42 });

        let temp_file = "/tmp/test_recorder.bin";
        recorder.save(temp_file).unwrap();

        let loaded = EventRecorder::load(temp_file).unwrap();
        assert_eq!(loaded.recordings().len(), 1);
        assert!(loaded.recordings()[0].event_type.contains("TestEvent"));

        std::fs::remove_file(temp_file).ok();
    }
}
