//! Concrete strategy implementations for securitization policies.

mod full_backing_issuance;
mod simple_collateral;

pub use full_backing_issuance::FullBackingIssuance;
pub use simple_collateral::SimpleCollateral;
