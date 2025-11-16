//! Game context for ISSUN
//!
//! GameContext holds persistent data that survives scene transitions.

/// Marker trait for game context
///
/// Game context should contain only data that:
/// - Persists across scene transitions
/// - Should be saved/loaded
/// - Is shared between scenes
pub trait GameContext: Send + Sync {
    // Marker trait - no required methods
}
