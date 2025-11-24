//! Event replayer implementation

use super::recorder::EventRecorder;
use super::types::{RecordedEvent, RecordingFile, RecordingStats};
use crate::event::{Event, EventBus};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// イベントデシリアライザー（型消去用）
pub trait EventDeserializer: Send + Sync {
    fn deserialize_and_publish(
        &self,
        payload: &[u8],
        event_bus: &mut EventBus,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// 型付きデシリアライザー
struct TypedEventDeserializer<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> TypedEventDeserializer<E> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> EventDeserializer for TypedEventDeserializer<E>
where
    E: Event + DeserializeOwned + serde::Serialize,
{
    fn deserialize_and_publish(
        &self,
        payload: &[u8],
        event_bus: &mut EventBus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event: E = bincode::deserialize(payload)?;
        event_bus.publish(event);
        Ok(())
    }
}

/// イベントリプレイヤー
pub struct EventReplayer {
    recordings: Vec<RecordedEvent>,
    current_frame: u64,
    current_index: usize,
    deserializers: HashMap<String, Box<dyn EventDeserializer>>,
}

impl EventReplayer {
    /// レコーダーからリプレイヤーを作成
    pub fn from_recorder(recorder: EventRecorder) -> Self {
        Self {
            recordings: recorder.recordings().to_vec(),
            current_frame: u64::MAX, // 初期値を最大値に（最初のフレームを確実に再生するため）
            current_index: 0,
            deserializers: HashMap::new(),
        }
    }

    /// ファイルから読み込み
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_handle = std::fs::File::open(path)?;
        let file: RecordingFile = bincode::deserialize_from(file_handle)?;

        Ok(Self {
            recordings: file.recordings,
            current_frame: u64::MAX, // 初期値を最大値に（最初のフレームを確実に再生するため）
            current_index: 0,
            deserializers: HashMap::new(),
        })
    }

    /// デシリアライザーを登録
    pub fn register_deserializer<E>(&mut self)
    where
        E: Event + DeserializeOwned + serde::Serialize + 'static,
    {
        let type_name = std::any::type_name::<E>().to_string();
        self.deserializers
            .insert(type_name, Box::new(TypedEventDeserializer::<E>::new()));
    }

    /// 指定フレームのイベントを再生
    pub fn replay_frame(
        &mut self,
        frame: u64,
        event_bus: &mut EventBus,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0;

        for recording in &self.recordings {
            if recording.frame == frame {
                if let Some(deserializer) = self.deserializers.get(&recording.event_type) {
                    deserializer.deserialize_and_publish(&recording.payload, event_bus)?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// 次のフレームを再生
    pub fn replay_next_frame(
        &mut self,
        event_bus: &mut EventBus,
    ) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        // 現在のインデックスから次のイベントを探す
        while self.current_index < self.recordings.len() {
            let recording = &self.recordings[self.current_index];
            let frame = recording.frame;

            // 現在のフレームと異なる場合、そのフレームを再生
            if frame != self.current_frame {
                self.current_frame = frame;
                event_bus.set_frame(frame);

                // このフレームの全イベントを再生
                let mut events_in_frame = vec![];
                for i in self.current_index..self.recordings.len() {
                    if self.recordings[i].frame == frame {
                        events_in_frame.push(i);
                    } else {
                        break;
                    }
                }

                for idx in events_in_frame {
                    let recording = &self.recordings[idx];
                    if let Some(deserializer) = self.deserializers.get(&recording.event_type) {
                        deserializer.deserialize_and_publish(&recording.payload, event_bus)?;
                    }
                    self.current_index = idx + 1;
                }

                return Ok(Some(frame));
            }

            self.current_index += 1;
        }

        Ok(None)
    }

    /// 全フレームを再生
    pub fn replay_all(
        &mut self,
        event_bus: &mut EventBus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while self.replay_next_frame(event_bus)?.is_some() {
            // Continue replaying
        }
        Ok(())
    }

    /// 特定フレーム範囲を再生
    pub fn replay_range(
        &mut self,
        start: u64,
        end: u64,
        event_bus: &mut EventBus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for frame in start..=end {
            self.replay_frame(frame, event_bus)?;
        }
        Ok(())
    }

    /// 現在のフレームを取得
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// 統計情報を取得
    pub fn stats(&self) -> RecordingStats {
        let mut stats = RecordingStats::new();

        for recording in &self.recordings {
            stats.add_event(&recording.event_type, recording.frame, recording.timestamp_ms);
        }

        stats
    }

    /// 記録されたイベント数を取得
    pub fn event_count(&self) -> usize {
        self.recordings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::recorder::EventRecorder;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct TestEvent {
        value: i32,
    }

    impl Event for TestEvent {}

    #[test]
    fn test_replayer_from_recorder() {
        let mut recorder = EventRecorder::new();
        recorder.start();
        recorder.record(&TestEvent { value: 42 });

        let replayer = EventReplayer::from_recorder(recorder);
        assert_eq!(replayer.event_count(), 1);
    }

    #[test]
    fn test_register_deserializer() {
        let recorder = EventRecorder::new();
        let mut replayer = EventReplayer::from_recorder(recorder);

        replayer.register_deserializer::<TestEvent>();

        // デシリアライザーが登録されているか確認
        assert!(replayer
            .deserializers
            .contains_key(std::any::type_name::<TestEvent>()));
    }

    #[test]
    fn test_replay_frame() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.set_frame(0);
        recorder.record(&TestEvent { value: 1 });

        recorder.set_frame(1);
        recorder.record(&TestEvent { value: 2 });

        let mut replayer = EventReplayer::from_recorder(recorder);
        replayer.register_deserializer::<TestEvent>();

        let mut bus = EventBus::new();

        // フレーム0を再生
        let count = replayer.replay_frame(0, &mut bus).unwrap();
        assert_eq!(count, 1);

        // フレーム1を再生
        let count = replayer.replay_frame(1, &mut bus).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_stats() {
        let mut recorder = EventRecorder::new();
        recorder.start();

        recorder.set_frame(0);
        recorder.record(&TestEvent { value: 1 });

        recorder.set_frame(1);
        recorder.record(&TestEvent { value: 2 });

        let replayer = EventReplayer::from_recorder(recorder);
        let stats = replayer.stats();

        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_frames, 2);
    }

    #[test]
    fn test_save_and_load() {
        let mut recorder = EventRecorder::new();
        recorder.start();
        recorder.record(&TestEvent { value: 42 });

        let temp_file = "/tmp/test_replayer.bin";
        recorder.save(temp_file).unwrap();

        let replayer = EventReplayer::load(temp_file).unwrap();
        assert_eq!(replayer.event_count(), 1);

        std::fs::remove_file(temp_file).ok();
    }
}
