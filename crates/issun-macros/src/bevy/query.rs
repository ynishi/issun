//! IssunQuery derive macro
//!
//! Auto-generates query helper methods that avoid borrowing conflicts.
//!
//! # Example
//! ```ignore
//! #[derive(IssunQuery)]
//! #[query(read = [ContagionInfection, ContagionNode])]
//! pub struct InfectionQuery;
//! ```
//!
//! This generates:
//! ```ignore
//! impl InfectionQuery {
//!     pub fn collect_states(world: &World) -> Vec<(Entity, InfectionState)> {
//!         world.iter_entities()
//!             .filter_map(|entity_ref| {
//!                 entity_ref.get::<ContagionInfection>()
//!                     .map(|inf| (entity_ref.id(), inf.state.clone()))
//!             })
//!             .collect()
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Attribute};

/// Derive macro for auto-generating query helper methods
pub fn derive_issun_query_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Parse #[query(read = [...])] attribute
    let query_attrs = parse_query_attributes(&input.attrs);

    if query_attrs.read_components.is_empty() {
        return syn::Error::new_spanned(
            &input,
            "IssunQuery requires #[query(read = [ComponentType, ...])] attribute"
        )
        .to_compile_error()
        .into();
    }

    // For now, generate a simple collect method for the first component
    // This can be extended to support multiple component combinations
    let primary_component = &query_attrs.read_components[0];

    let expanded = quote! {
        impl #struct_name {
            /// Collect all entities with the primary component
            #[allow(dead_code)]
            pub fn collect(world: &::bevy::prelude::World) -> ::std::vec::Vec<::bevy::prelude::Entity> {
                world.iter_entities()
                    .filter(|entity_ref| entity_ref.contains::<#primary_component>())
                    .map(|entity_ref| entity_ref.id())
                    .collect()
            }

            /// Count entities with the primary component
            #[allow(dead_code)]
            pub fn count(world: &::bevy::prelude::World) -> usize {
                world.iter_entities()
                    .filter(|entity_ref| entity_ref.contains::<#primary_component>())
                    .count()
            }

            /// Iterate over entities with the primary component
            #[allow(dead_code)]
            pub fn for_each<F>(world: &::bevy::prelude::World, mut f: F)
            where
                F: FnMut(::bevy::prelude::Entity, &#primary_component),
            {
                for entity_ref in world.iter_entities() {
                    if let Some(component) = entity_ref.get::<#primary_component>() {
                        f(entity_ref.id(), component);
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

struct QueryAttributes {
    read_components: Vec<syn::Type>,
}

/// Parse #[query(...)] attributes
fn parse_query_attributes(attrs: &[Attribute]) -> QueryAttributes {
    let mut read_components = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("query") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("read") {
                // Parse the value which should be a list like [Type1, Type2]
                let value = meta.value()?;
                let content: syn::ExprArray = value.parse()?;

                for expr in content.elems {
                    if let syn::Expr::Path(path_expr) = expr {
                        let ty: syn::Type = syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: path_expr.path,
                        });
                        read_components.push(ty);
                    }
                }
                Ok(())
            } else {
                Err(meta.error("expected `read = [...]`"))
            }
        });
    }

    QueryAttributes { read_components }
}
