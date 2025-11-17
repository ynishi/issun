//! Inventory types and traits

/// Item trait for inventory management
///
/// Implement this trait for game items that can be stored in inventories.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Debug)]
/// pub struct Weapon {
///     pub name: String,
///     pub attack: i32,
/// }
///
/// impl Item for Weapon {}
/// ```
pub trait Item: Clone + Send + Sync + 'static {}

/// Auto-implement Item for any type that satisfies the bounds
impl<T> Item for T where T: Clone + Send + Sync + 'static {}
