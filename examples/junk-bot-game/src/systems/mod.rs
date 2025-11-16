//! Business logic layer
//!
//! Domain services for complex logic that doesn't fit in Entities or SceneHandler.
//! Keep this minimal - most game logic should live in:
//! - entities/* (data + factory methods + simple domain logic)
//! - scene_handler.rs (game flow + scene transitions)
//!
//! Only add services here when:
//! - Logic is too complex for Entity methods
//! - External API clients are needed
//! - Cross-entity orchestration is required
