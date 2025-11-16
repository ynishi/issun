//! Proc macros for ISSUN game engine
//!
//! This crate provides derive macros for:
//! - `#[derive(Scene)]` - Auto-implement Scene trait
//! - `#[derive(Entity)]` - Auto-generate entity methods
//! - `#[derive(Asset)]` - Auto-generate asset loading

use proc_macro::TokenStream;

/// Derive macro for Scene trait
///
/// # Example
/// ```ignore
/// #[derive(Scene)]
/// #[scene(context = "GameContext")]
/// enum MyGameScene {
///     Title,
///     Combat(CombatData),
/// }
/// ```
#[proc_macro_derive(Scene, attributes(scene))]
pub fn derive_scene(_input: TokenStream) -> TokenStream {
    // TODO: Implement Scene derive macro
    TokenStream::new()
}

/// Derive macro for Entity trait
///
/// # Example
/// ```ignore
/// #[derive(Entity)]
/// pub struct Player {
///     #[stat(max = 100)]
///     pub hp: i32,
/// }
/// ```
#[proc_macro_derive(Entity, attributes(stat, position))]
pub fn derive_entity(_input: TokenStream) -> TokenStream {
    // TODO: Implement Entity derive macro
    TokenStream::new()
}

/// Derive macro for Asset trait
///
/// # Example
/// ```ignore
/// #[derive(Asset)]
/// pub struct EnemyAsset {
///     pub name: &'static str,
///     pub hp: i32,
/// }
/// ```
#[proc_macro_derive(Asset)]
pub fn derive_asset(_input: TokenStream) -> TokenStream {
    // TODO: Implement Asset derive macro
    TokenStream::new()
}
