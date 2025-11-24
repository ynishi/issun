//! Event recording and replay for deterministic reproduction
//!
//! This module provides tools to record gameplay events and replay them
//! deterministically for debugging, testing, and analysis.
//!
//! # Example
//!
//! ```ignore
//! use issun::replay::{EventRecorder, EventReplayer};
//!
//! // Recording
//! let mut recorder = EventRecorder::new();
//! recorder.start();
//!
//! // ... game runs, events are recorded ...
//!
//! recorder.save("gameplay.replay")?;
//!
//! // Replay
//! let mut replayer = EventReplayer::load("gameplay.replay")?;
//! replayer.register_deserializer::<MyEvent>();
//! replayer.replay_all(&mut event_bus)?;
//! ```

pub mod recorder;
pub mod replayer;
pub mod types;

pub use recorder::EventRecorder;
pub use replayer::{EventDeserializer, EventReplayer};
pub use types::{RecordedEvent, RecordingFile, RecordingMetadata, RecordingStats};
