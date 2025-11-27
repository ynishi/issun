/// Linear influence strategy implementation.
pub mod linear_influence;
/// Simple context strategy implementation.
pub mod simple_context;
/// Skeptical resistance strategy implementation.
pub mod skeptical_resistance;

pub use linear_influence::LinearInfluence;
pub use simple_context::NoContext;
pub use skeptical_resistance::SkepticalResistance;
