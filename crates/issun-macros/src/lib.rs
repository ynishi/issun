//! Proc macros for ISSUN game engine
//!
//! This crate provides derive macros for:
//! - `#[derive(Scene)]` - Auto-implement Scene trait
//! - `#[derive(Entity)]` - Auto-generate entity methods
//! - `#[derive(Asset)]` - Auto-generate asset loading

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for Scene trait
///
/// # Example
/// ```ignore
/// #[derive(Scene)]
/// enum MyGameScene {
///     Title,
///     Combat(CombatData),
/// }
/// ```
#[proc_macro_derive(Scene, attributes(scene))]
pub fn derive_scene(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // For now, implement a default Scene that returns Stay on update
    // In the future, we can support enum-based scene routing
    let expanded = quote! {
        #[async_trait::async_trait]
        impl Scene for #name {
            async fn on_enter(&mut self) {
                // Default implementation: do nothing
            }

            async fn on_update(&mut self) -> SceneTransition {
                // Default implementation: stay in current scene
                SceneTransition::Stay
            }

            async fn on_exit(&mut self) {
                // Default implementation: do nothing
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Entity trait
///
/// # Example
/// ```ignore
/// #[derive(Entity)]
/// pub struct Player {
///     #[entity(id)]
///     pub name: String,
///     pub hp: i32,
/// }
/// ```
#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Extract fields to find #[entity(id)]
    let id_field = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    // Find field with #[entity(id)] attribute
                    fields.named.iter()
                        .find(|field| {
                            field.attrs.iter().any(|attr| {
                                attr.path().is_ident("entity")
                            })
                        })
                        .map(|field| field.ident.as_ref().unwrap())
                },
                _ => None,
            }
        },
        _ => None,
    };

    let id_impl = if let Some(field_name) = id_field {
        quote! {
            fn id(&self) -> &str {
                &self.#field_name
            }
        }
    } else {
        // No #[entity(id)] found - use default implementation
        quote! {
            fn id(&self) -> &str {
                ""
            }
        }
    };

    let expanded = quote! {
        #[async_trait::async_trait]
        impl Entity for #name {
            #id_impl

            async fn update(&mut self, _ctx: &mut Context) {
                // Default implementation: do nothing
            }
        }
    };

    TokenStream::from(expanded)
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
pub fn derive_asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl Asset for #name {
            // Uses default implementation from trait
        }
    };

    TokenStream::from(expanded)
}
