//! Proc macros for ISSUN game engine
//!
//! This crate provides derive macros for:
//! - `#[derive(Scene)]` - Auto-implement Scene trait
//! - `#[derive(Entity)]` - Auto-generate entity methods
//! - `#[derive(Asset)]` - Auto-generate asset loading

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit};

/// Derive macro for Scene trait
///
/// # Example
/// ```ignore
/// #[derive(Scene)]
/// #[scene(context = "GameContext", initial = "Title(TitleSceneData::new())")]
/// enum MyGameScene {
///     Title(TitleSceneData),
///     Combat(CombatData),
/// }
/// ```
///
/// This will auto-generate:
/// - Scene trait implementation
/// - GameState struct (or custom name via `name` attribute)
/// - GameState::new() with initial scene and context
#[proc_macro_derive(Scene, attributes(scene))]
pub fn derive_scene(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let scene_name = &input.ident;

    // Parse #[scene(...)] attributes
    let scene_attrs = parse_scene_attributes(&input.attrs);

    // Generate Scene trait implementation
    // Note: We rely on Scene and SceneTransition being in scope via `use issun::prelude::*`
    let scene_impl = quote! {
        #[::async_trait::async_trait]
        impl ::issun::scene::Scene for #scene_name {
            async fn on_enter(&mut self) {
                // Default implementation: do nothing
            }

            async fn on_update(&mut self) -> ::issun::scene::SceneTransition {
                // Default implementation: stay in current scene
                ::issun::scene::SceneTransition::Stay
            }

            async fn on_exit(&mut self) {
                // Default implementation: do nothing
            }
        }
    };

    // Generate GameState struct if context and initial are specified
    let game_state_gen = if let (Some(context), Some(initial)) = (&scene_attrs.context, &scene_attrs.initial) {
        let state_name = scene_attrs.name.as_ref()
            .map(|n| format_ident!("{}", n))
            .unwrap_or_else(|| format_ident!("GameState"));

        let context_ident = format_ident!("{}", context);
        let initial_expr: proc_macro2::TokenStream = initial.parse().unwrap();

        quote! {
            /// Auto-generated game state combining scene and context
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct #state_name {
                pub scene: #scene_name,
                pub ctx: #context_ident,
                pub should_quit: bool,
            }

            impl #state_name {
                pub fn new() -> Self {
                    Self {
                        scene: #scene_name::#initial_expr,
                        ctx: #context_ident::new(),
                        should_quit: false,
                    }
                }
            }

            impl Default for #state_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    } else {
        quote! {}
    };

    // Generate handler dispatcher if handler_params is specified
    let handler_gen = if let Some(params) = &scene_attrs.handler_params {
        let handler_name = scene_attrs.handler.as_ref()
            .map(|h| format_ident!("{}", h))
            .unwrap_or_else(|| format_ident!("handle_input"));

        let params_tokens: proc_macro2::TokenStream = params.parse().unwrap();

        // Extract parameter names from "ctx: &mut GameContext, input: InputEvent" -> ["ctx", "input"]
        let param_names = extract_param_names(params);

        // Default return type based on scene name
        let return_type = scene_attrs.handler_return.as_ref()
            .map(|r| r.parse::<proc_macro2::TokenStream>().unwrap())
            .unwrap_or_else(|| {
                quote! { (#scene_name, ::issun::scene::SceneTransition) }
            });

        // Extract variants from enum
        let variants = if let Data::Enum(data_enum) = &input.data {
            &data_enum.variants
        } else {
            panic!("Scene derive macro only works on enums");
        };

        // Generate match arms for each variant
        let match_arms = variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            quote! {
                #scene_name::#variant_name(data) => data.#handler_name(#(#param_names),*)
            }
        });

        quote! {
            /// Auto-generated scene input handler dispatcher
            pub fn handle_scene_input(
                scene: #scene_name,
                #params_tokens,
            ) -> #return_type {
                match scene {
                    #(#match_arms),*
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #scene_impl
        #game_state_gen
        #handler_gen
    };

    TokenStream::from(expanded)
}

/// Parse #[scene(...)] attributes
struct SceneAttributes {
    context: Option<String>,
    initial: Option<String>,
    name: Option<String>,
    handler: Option<String>,
    handler_params: Option<String>,
    handler_return: Option<String>,
}

fn parse_scene_attributes(attrs: &[syn::Attribute]) -> SceneAttributes {
    let mut context = None;
    let mut initial = None;
    let mut name = None;
    let mut handler = None;
    let mut handler_params = None;
    let mut handler_return = None;

    for attr in attrs {
        if attr.path().is_ident("scene") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("context") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<Lit>() {
                            if let Lit::Str(s) = lit {
                                context = Some(s.value());
                            }
                        }
                    }
                } else if meta.path.is_ident("initial") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<Lit>() {
                            if let Lit::Str(s) = lit {
                                initial = Some(s.value());
                            }
                        }
                    }
                } else if meta.path.is_ident("name") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<Lit>() {
                            if let Lit::Str(s) = lit {
                                name = Some(s.value());
                            }
                        }
                    }
                } else if meta.path.is_ident("handler") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<Lit>() {
                            if let Lit::Str(s) = lit {
                                handler = Some(s.value());
                            }
                        }
                    }
                } else if meta.path.is_ident("handler_params") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<Lit>() {
                            if let Lit::Str(s) = lit {
                                handler_params = Some(s.value());
                            }
                        }
                    }
                } else if meta.path.is_ident("handler_return") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<Lit>() {
                            if let Lit::Str(s) = lit {
                                handler_return = Some(s.value());
                            }
                        }
                    }
                }
                Ok(())
            });
        }
    }

    SceneAttributes {
        context,
        initial,
        name,
        handler,
        handler_params,
        handler_return,
    }
}

/// Extract parameter names from function parameters string
/// "ctx: &mut GameContext, input: InputEvent" -> vec![ident("ctx"), ident("input")]
fn extract_param_names(params_str: &str) -> Vec<proc_macro2::Ident> {
    params_str
        .split(',')
        .filter_map(|param| {
            let param = param.trim();
            // Extract the identifier before the colon
            param.split(':').next().map(|name| {
                let name = name.trim();
                format_ident!("{}", name)
            })
        })
        .collect()
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
