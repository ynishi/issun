//! Event chain tracing for debugging and visualization
//!
//! This module provides tools to trace Event→Hook call chains and generate
//! visual graphs (Mermaid, Graphviz) for debugging complex event flows.
//!
//! # Example
//!
//! ```
//! use issun::trace::EventChainTracer;
//!
//! let mut tracer = EventChainTracer::new();
//! tracer.enable();
//!
//! // ... game runs, events are traced ...
//!
//! // Generate Mermaid graph
//! let mermaid = tracer.generate_mermaid_graph();
//! std::fs::write("event_chain.mmd", mermaid).unwrap();
//! ```

pub mod generator;
pub mod macros;
pub mod tracer;
pub mod types;

pub use tracer::{EventChainTracer, TracerStats};
pub use types::{HookResult, TraceEntry, TraceEntryType};
