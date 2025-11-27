//! IssunEntity derive macro
//!
//! Auto-generates component getter methods for any Resource with an Entity field, avoiding borrowing errors.
//!
//! # Example
//! ```ignore
//! #[derive(Resource, IssunEntity)]
//! #[components(ActionPoints, Health, ContagionInfection)]
//! pub struct Player {
//!     #[primary]
//!     pub entity: Entity,
//! }
//! ```
//!
//! This generates:
//! ```ignore
//! impl Player {
//!     pub fn action_points<'w>(&self, world: &'w World) -> Option<&'w ActionPoints> {
//!         world.get::<ActionPoints>(self.entity)
//!     }
//!
//!     pub fn action_points_mut<'w>(&self, world: &'w mut World) -> Option<Mut<'w, ActionPoints>> {
//!         world.get_mut::<ActionPoints>(self.entity)
//!     }
//!
//!     pub fn health<'w>(&self, world: &'w World) -> Option<&'w Health> { ... }
//!     pub fn health_mut<'w>(&self, world: &'w mut World) -> Option<Mut<'w, Health>> { ... }
//!     // ... other components
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Attribute, Type};

/// Derive macro for auto-generating entity component getters
pub fn derive_issun_entity_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Find the field marked with #[primary] attribute
    let primary_field = find_primary_field(&input);

    if primary_field.is_none() {
        return syn::Error::new_spanned(
            &input,
            "IssunEntity requires a field marked with #[primary]"
        )
        .to_compile_error()
        .into();
    }

    let primary_field = primary_field.unwrap();

    // Parse #[components(...)] attribute to get component types
    let component_types = parse_components_attr(&input.attrs);

    let getter_methods: Vec<_> = component_types
        .iter()
        .map(|component_ty| {
            // Extract the simple type name for method naming
            let type_name = extract_type_name(component_ty);
            let method_name = format_ident!("{}", to_snake_case(&type_name));
            let method_name_mut = format_ident!("{}_mut", to_snake_case(&type_name));

            quote! {
                #[allow(dead_code)]
                pub fn #method_name<'w>(&self, world: &'w ::bevy::prelude::World)
                    -> ::std::option::Option<&'w #component_ty>
                {
                    world.get::<#component_ty>(self.#primary_field)
                }

                #[allow(dead_code)]
                pub fn #method_name_mut<'w>(&self, world: &'w mut ::bevy::prelude::World)
                    -> ::std::option::Option<::bevy::prelude::Mut<'w, #component_ty>>
                {
                    world.get_mut::<#component_ty>(self.#primary_field)
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #struct_name {
            #(#getter_methods)*
        }
    };

    TokenStream::from(expanded)
}

/// Find the field marked with #[primary] attribute
fn find_primary_field(input: &DeriveInput) -> Option<&syn::Ident> {
    match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    fields.named.iter().find_map(|field| {
                        if has_primary_attr(field) {
                            field.ident.as_ref()
                        } else {
                            None
                        }
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if a field has #[primary] attribute
fn has_primary_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("primary")
    })
}

/// Parse #[components(...)] attribute to extract component types
fn parse_components_attr(attrs: &[Attribute]) -> Vec<Type> {
    for attr in attrs {
        if !attr.path().is_ident("components") {
            continue;
        }

        // Parse the content inside #[components(...)]
        let result = attr.parse_args_with(|input: syn::parse::ParseStream| {
            let mut types = Vec::new();

            // Parse comma-separated list of types
            loop {
                if input.is_empty() {
                    break;
                }

                let ty: Type = input.parse()?;
                types.push(ty);

                // Check for comma
                if input.peek(syn::Token![,]) {
                    let _: syn::Token![,] = input.parse()?;
                } else {
                    break;
                }
            }

            Ok(types)
        });

        if let Ok(types) = result {
            return types;
        }
    }

    // Default: ActionPoints if no #[components(...)] attribute
    vec![syn::parse_str("ActionPoints").unwrap()]
}

/// Extract the simple type name from a Type (for method naming)
fn extract_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

// Import format_ident from quote
use quote::format_ident;
