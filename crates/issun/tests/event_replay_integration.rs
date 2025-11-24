//! Integration tests for event recording and replay

use issun::event::{Event, EventBus};
use issun::replay::{EventRecorder, EventReplayer};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct TestEvent {
    id: u32,
    message: String,
}

impl Event for TestEvent {}

#[test]
fn test_event_bus_recording_disabled_by_default() {
    let mut bus = EventBus::new();

    bus.publish(TestEvent {
        id: 1,
        message: "test".to_string(),
    });

    // レコーダーが設定されていない場合、エラーにならない
    assert_eq!(bus.current_frame(), 0);
}

#[test]
fn test_event_bus_set_recorder() {
    let mut bus = EventBus::new();
    let mut recorder = EventRecorder::new();
    recorder.start();

    let recorder = Arc::new(Mutex::new(recorder));
    bus.set_recorder(recorder.clone());

    // イベントをpublish
    bus.publish(TestEvent {
        id: 1,
        message: "test".to_string(),
    });

    // 記録が取れているか確認
    let r = recorder.lock().unwrap();
    let recordings = r.recordings();

    assert_eq!(recordings.len(), 1);
    assert!(recordings[0].event_type.contains("TestEvent"));
}

#[test]
fn test_event_bus_recording_with_frames() {
    let mut bus = EventBus::new();
    let mut recorder = EventRecorder::new();
    recorder.start();

    let recorder = Arc::new(Mutex::new(recorder));
    bus.set_recorder(recorder.clone());

    // フレーム0
    bus.set_frame(0);
    bus.publish(TestEvent {
        id: 1,
        message: "frame 0".to_string(),
    });

    // フレーム1
    bus.set_frame(1);
    bus.publish(TestEvent {
        id: 2,
        message: "frame 1".to_string(),
    });

    // フレーム2
    bus.set_frame(2);
    bus.publish(TestEvent {
        id: 3,
        message: "frame 2".to_string(),
    });

    // 記録を確認
    let r = recorder.lock().unwrap();
    let recordings = r.recordings();

    assert_eq!(recordings.len(), 3);
    assert_eq!(recordings[0].frame, 0);
    assert_eq!(recordings[1].frame, 1);
    assert_eq!(recordings[2].frame, 2);
}

#[test]
fn test_record_and_replay() {
    // === Recording ===
    let mut bus = EventBus::new();
    let mut recorder = EventRecorder::new();
    recorder.start();

    let recorder = Arc::new(Mutex::new(recorder));
    bus.set_recorder(recorder.clone());

    // イベントをpublish
    for i in 0..5 {
        bus.set_frame(i);
        bus.publish(TestEvent {
            id: i as u32,
            message: format!("event {}", i),
        });
    }

    // === Replay ===
    // レコーダーからファイルに保存してからリプレイヤーで読み込む
    let temp_file = "/tmp/test_record_and_replay.bin";
    recorder.lock().unwrap().save(temp_file).unwrap();

    let mut replayer = EventReplayer::load(temp_file).unwrap();
    replayer.register_deserializer::<TestEvent>();

    let mut replay_bus = EventBus::new();

    // フレーム0を再生
    let count = replayer.replay_frame(0, &mut replay_bus).unwrap();
    assert_eq!(count, 1);

    // フレーム1を再生
    let count = replayer.replay_frame(1, &mut replay_bus).unwrap();
    assert_eq!(count, 1);

    std::fs::remove_file(temp_file).ok();
}

#[test]
fn test_save_and_load_replay() {
    // === Recording ===
    let mut bus = EventBus::new();
    let mut recorder = EventRecorder::new();
    recorder.start();

    let recorder = Arc::new(Mutex::new(recorder));
    bus.set_recorder(recorder.clone());

    // イベントをpublish
    for i in 0..3 {
        bus.set_frame(i);
        bus.publish(TestEvent {
            id: i as u32,
            message: format!("event {}", i),
        });
    }

    // 保存
    let temp_file = "/tmp/test_replay.bin";
    recorder.lock().unwrap().save(temp_file).unwrap();

    // === Load and Replay ===
    let mut replayer = EventReplayer::load(temp_file).unwrap();
    replayer.register_deserializer::<TestEvent>();

    assert_eq!(replayer.event_count(), 3);

    let stats = replayer.stats();
    assert_eq!(stats.total_events, 3);
    assert_eq!(stats.total_frames, 3);

    std::fs::remove_file(temp_file).ok();
}

#[test]
fn test_replay_next_frame() {
    // === Recording ===
    let mut bus = EventBus::new();
    let mut recorder = EventRecorder::new();
    recorder.start();

    let recorder = Arc::new(Mutex::new(recorder));
    bus.set_recorder(recorder.clone());

    // イベントをpublish
    for i in 0..3 {
        bus.set_frame(i);
        bus.publish(TestEvent {
            id: i as u32,
            message: format!("event {}", i),
        });
    }

    // === Replay ===
    let temp_file = "/tmp/test_replay_next_frame.bin";
    recorder.lock().unwrap().save(temp_file).unwrap();

    let mut replayer = EventReplayer::load(temp_file).unwrap();
    replayer.register_deserializer::<TestEvent>();

    let mut replay_bus = EventBus::new();

    // フレームを順次再生
    let frame = replayer.replay_next_frame(&mut replay_bus).unwrap();
    assert_eq!(frame, Some(0));

    let frame = replayer.replay_next_frame(&mut replay_bus).unwrap();
    assert_eq!(frame, Some(1));

    let frame = replayer.replay_next_frame(&mut replay_bus).unwrap();
    assert_eq!(frame, Some(2));

    // 終了
    let frame = replayer.replay_next_frame(&mut replay_bus).unwrap();
    assert_eq!(frame, None);

    std::fs::remove_file(temp_file).ok();
}

#[test]
fn test_clear_recorder() {
    let mut bus = EventBus::new();
    let mut recorder = EventRecorder::new();
    recorder.start();

    let recorder = Arc::new(Mutex::new(recorder));
    bus.set_recorder(recorder.clone());

    // イベントをpublish
    bus.publish(TestEvent {
        id: 1,
        message: "test".to_string(),
    });

    // レコーダーをクリア
    bus.clear_recorder();

    // 新しいイベントは記録されない
    bus.publish(TestEvent {
        id: 2,
        message: "test2".to_string(),
    });

    // 記録は1つだけ
    let r = recorder.lock().unwrap();
    assert_eq!(r.recordings().len(), 1);
}
