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
pub fn derive_asset(_input: TokenStream) -> TokenStream {
    // TODO: Implement Asset derive macro
    TokenStream::new()
}
