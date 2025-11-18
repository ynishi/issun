//! Business logic layer
//!
//! Systems orchestrate stateful workflows by invoking stateless Services and
//! mutating `GameContext`. Keep systems small and focused: let entities own
//! primitive data/logic, scenes coordinate flow, and systems handle sequences.

pub mod ping_pong;
