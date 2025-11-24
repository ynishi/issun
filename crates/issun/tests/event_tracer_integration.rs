//! Integration tests for EventBus tracing

use issun::event::{Event, EventBus};
use issun::trace::{EventChainTracer, TraceEntryType};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct TestEvent {
    id: u32,
    message: String,
}

impl Event for TestEvent {}

#[test]
fn test_event_bus_tracing_disabled_by_default() {
    let mut bus = EventBus::new();

    bus.publish(TestEvent {
        id: 1,
        message: "test".to_string(),
    });

    // トレーサーが設定されていない場合、エラーにならない
    assert_eq!(bus.current_frame(), 0);
}

#[test]
fn test_event_bus_set_tracer() {
    let mut bus = EventBus::new();
    let mut tracer = EventChainTracer::new();
    tracer.enable();

    let tracer = Arc::new(Mutex::new(tracer));
    bus.set_tracer(tracer.clone());

    // イベントをpublish
    bus.publish(TestEvent {
        id: 1,
        message: "test".to_string(),
    });

    // トレースが記録されているか確認
    let t = tracer.lock().unwrap();
    let traces = t.traces();

    assert_eq!(traces.len(), 1);

    match &traces[0].entry_type {
        TraceEntryType::EventPublished { event_type, .. } => {
            assert!(event_type.contains("TestEvent"));
        }
        _ => panic!("Expected EventPublished"),
    }
}

#[test]
fn test_event_bus_tracing_with_frames() {
    let mut bus = EventBus::new();
    let mut tracer = EventChainTracer::new();
    tracer.enable();

    let tracer = Arc::new(Mutex::new(tracer));
    bus.set_tracer(tracer.clone());

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

    // トレースを確認
    let t = tracer.lock().unwrap();
    let traces = t.traces();

    assert_eq!(traces.len(), 3);
    assert_eq!(traces[0].frame, 0);
    assert_eq!(traces[1].frame, 1);
    assert_eq!(traces[2].frame, 2);
}

#[test]
fn test_event_bus_clear_tracer() {
    let mut bus = EventBus::new();
    let mut tracer = EventChainTracer::new();
    tracer.enable();

    let tracer = Arc::new(Mutex::new(tracer));
    bus.set_tracer(tracer.clone());

    // イベントをpublish
    bus.publish(TestEvent {
        id: 1,
        message: "test".to_string(),
    });

    // トレーサーをクリア
    bus.clear_tracer();

    // 新しいイベントはトレースされない
    bus.publish(TestEvent {
        id: 2,
        message: "test2".to_string(),
    });

    // トレースは1つだけ
    let t = tracer.lock().unwrap();
    assert_eq!(t.traces().len(), 1);
}

#[test]
fn test_event_bus_tracing_multiple_event_types() {
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct Event1 {
        value: i32,
    }
    impl Event for Event1 {}

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct Event2 {
        name: String,
    }
    impl Event for Event2 {}

    let mut bus = EventBus::new();
    let mut tracer = EventChainTracer::new();
    tracer.enable();

    let tracer = Arc::new(Mutex::new(tracer));
    bus.set_tracer(tracer.clone());

    // 異なる型のイベントをpublish
    bus.publish(Event1 { value: 42 });
    bus.publish(Event2 {
        name: "test".to_string(),
    });
    bus.publish(Event1 { value: 99 });

    // トレースを確認
    let t = tracer.lock().unwrap();
    let traces = t.traces();

    assert_eq!(traces.len(), 3);

    match &traces[0].entry_type {
        TraceEntryType::EventPublished { event_type, .. } => {
            assert!(event_type.contains("Event1"));
        }
        _ => panic!("Expected EventPublished"),
    }

    match &traces[1].entry_type {
        TraceEntryType::EventPublished { event_type, .. } => {
            assert!(event_type.contains("Event2"));
        }
        _ => panic!("Expected EventPublished"),
    }
}

#[test]
fn test_generate_graph_from_traced_events() {
    let mut bus = EventBus::new();
    let mut tracer = EventChainTracer::new();
    tracer.enable();

    let tracer = Arc::new(Mutex::new(tracer));
    bus.set_tracer(tracer.clone());

    // 複数のイベントをpublish
    for i in 0..5 {
        bus.set_frame(i);
        bus.publish(TestEvent {
            id: i as u32,
            message: format!("event {}", i),
        });
    }

    // Mermaidグラフを生成
    let t = tracer.lock().unwrap();
    let mermaid = t.generate_mermaid_graph();

    // グラフが有効な内容を含むか確認
    assert!(mermaid.contains("graph TD"));
    assert!(mermaid.contains("TestEvent"));
    assert!(mermaid.contains("EventBus"));
}
