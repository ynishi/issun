//! Strategy implementations for rights policies.

pub mod recognition;
pub mod rights_system;
pub mod transfer;

// Re-export strategies
pub use recognition::{AuthorityRecognition, ConsensusRecognition, SelfRecognition};
pub use rights_system::{AbsoluteRights, LayeredRights, PartialRights};
pub use transfer::{FreeTransfer, NonTransferable, RestrictedTransfer};
